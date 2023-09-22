use std::{
    collections::HashMap,
    fmt::Display,
    ops::Deref,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{models::AnchorInfo, AppWindow};
use crate::{util::vec_string_to_slint_model, *};
use futures::channel::mpsc::Receiver;
use log::{debug, error, info};
use slint::{ComponentHandle, Image, ModelRc, SharedPixelBuffer, VecModel};
use tokio::runtime::{self, Runtime};

pub struct Server {
    inner: Inner,
}

impl Server {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            inner: Inner {
                msg_rx: Arc::new(Mutex::new(rx)),
                msg_tx: Arc::new(Mutex::new(tx)),
                anchor_mgr: Arc::new(tokio::sync::Mutex::new(AnchorMgr::new())),
            },
        }
    }
    pub fn get_msg_tx(&self) -> std::sync::mpsc::Sender<Msg> {
        return self.inner.msg_tx.lock().expect("lock").clone();
    }
    pub fn push_msg(&self, msg: Msg) -> anyhow::Result<()> {
        Ok(self
            .inner
            .msg_tx
            .lock()
            .expect("lock")
            .send(msg)
            .expect("send"))
    }

    pub async fn start_loop(&self, ui_weak: slint::Weak<AppWindow>) {
        info!("server start ...");
        // 加载所有订阅主播
        let mut flush_rx = self.auto_flush().await;
        let mut msg_rx = self.get_msg_recv().await;
        loop {
            tokio::select! {

            Some(msg) = msg_rx.recv() => {
                debug!("recv msg {:?}", msg);

                match msg {
                    Msg::Follow(plt, rid) => {
                        let anchor_info = match AnchorInfo::get(&plt, &rid).await {
                            Ok(v) => v,
                            Err(e) => {
                                error!("get err {}", e);
                                continue;
                            }
                        };
                        if let Err(e) = self
                            .inner
                            .anchor_mgr
                            .lock()
                            .await
                            .add_living(anchor_info)
                            .await
                        {
                            error!("add anchor err: {:?}", e);
                            continue;
                        }

                        let _ = self.inner.anchor_mgr.lock().await.save_anchor().await;

                        self.inner
                            .anchor_mgr
                            .lock()
                            .await
                            .update_ui(ui_weak.clone())
                            .await;
                    }
                    Msg::Exit => {
                        info!("server exit ...");
                        return;
                    }
                    Msg::Play(urls) => {
                        for url in urls {
                            _ = util::play(url).await;
                            break;
                        }
                    }
                }
            }
            Some(anchor) = flush_rx.recv() => {
                if let Err(e) = self
                            .inner
                            .anchor_mgr
                            .lock()
                            .await
                            .add_living(anchor)
                            .await
                        {
                            error!("add anchor err: {:?}", e);
                            continue;
                        }
                        self.inner
                        .anchor_mgr
                        .lock()
                        .await
                        .update_ui(ui_weak.clone())
                        .await;

            }
            }
        }
    }

    async fn get_msg_recv(&self) -> tokio::sync::mpsc::Receiver<Msg> {
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        let msg_rx = self.inner.msg_rx.clone();
        tokio::spawn(async move {
            loop {
                let msg_rx = msg_rx.clone();
                let r: anyhow::Result<Msg> = tokio::task::spawn_blocking(move || {
                    let guard = msg_rx.lock().expect("lock");
                    Ok(guard.recv().expect("recv err"))
                })
                .await
                .expect("spawn_blocking");
                match r {
                    Ok(msg) => {
                        _ = tx.send(msg).await;
                    }
                    Err(_) => break,
                }
            }
        });

        rx
    }

    async fn auto_flush(&self) -> tokio::sync::mpsc::Receiver<AnchorInfo> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let mgr = self.inner.anchor_mgr.clone();
        mgr.lock().await.load_config().await.expect("load config");
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(60));
            loop {
                tick.tick().await;

                let all = mgr.lock().await.get_all_config();

                for anchor in all.iter() {
                    match AnchorInfo::get(&anchor.platform, &anchor.rid).await {
                        Ok(v) => {
                            _ = tx.send(v).await;
                        }
                        Err(e) => {
                            error!("get err {}", e);
                            continue;
                        }
                    };
                }
            }
        });

        return rx;
    }
}

struct Inner {
    pub msg_rx: Arc<Mutex<std::sync::mpsc::Receiver<Msg>>>,
    pub msg_tx: Arc<Mutex<std::sync::mpsc::Sender<Msg>>>,
    pub anchor_mgr: Arc<tokio::sync::Mutex<AnchorMgr>>,
}

#[derive(Debug)]
pub enum Msg {
    Follow(String, String),
    Play(Vec<String>),
    Exit,
}

pub struct AnchorMgr {
    anchor_map: HashMap<(String, String), AnchorInfo>, // 开播列表
    config_map: HashMap<(String, String), AnchorInfo>, // 配置文件
}

impl AnchorMgr {
    pub fn new() -> Self {
        Self {
            anchor_map: HashMap::new(),
            config_map: HashMap::new(),
        }
    }

    // 加载配置
    pub async fn load_config(&mut self) -> anyhow::Result<()> {
        let anchors = config::load_anchor().await?;
        info!("load {} from config", anchors.len());
        for anchor in anchors {
            self.config_map.insert(Self::key(&anchor), anchor); // 初始化配置
        }
        return Ok(());
    }

    // 配置写入到文件
    pub async fn save_anchor(&self) -> anyhow::Result<()> {
        let anchors: Vec<AnchorInfo> = self.config_map.iter().map(|(_, v)| v.clone()).collect();
        config::save_anchor(&anchors).await?;
        Ok(())
    }

    // 添加目前的开播列表 直播中的
    pub async fn add_living(&mut self, mut anchor: AnchorInfo) -> anyhow::Result<()> {
        let cover = cache::download_img_to_cache(&anchor.cover).await?;
        let head = cache::download_img_to_cache(&anchor.head).await?;
        anchor.cover_path = Some(cover);
        anchor.head_path = Some(head);
        self.anchor_map.insert(Self::key(&anchor), anchor.clone());
        self.config_map.insert(Self::key(&anchor), anchor); // 配置列表同步添加
        Ok(())
    }

    // 添加配置，不要求开播
    pub async fn add_config(&mut self, anchor: AnchorInfo) {
        self.config_map.insert(Self::key(&anchor), anchor); // 配置列表同步添加
    }

    pub fn remove(&mut self, anchor: &AnchorInfo) {
        self.anchor_map.remove(&Self::key(anchor));
        self.config_map.remove(&Self::key(anchor));
    }

    fn key(anchor: &AnchorInfo) -> (String, String) {
        (anchor.platform.clone(), anchor.rid.clone())
    }

    pub fn get_all_living(&self) -> Vec<AnchorInfo> {
        let mut ret = vec![];
        for (k, v) in self.anchor_map.iter() {
            ret.push(v.clone());
        }
        ret
    }

    pub fn get_all_config(&self) -> Vec<AnchorInfo> {
        let mut ret = vec![];
        for (k, v) in self.config_map.iter() {
            ret.push(v.clone());
        }
        ret
    }

    async fn update_ui(&self, ui_weak: slint::Weak<AppWindow>) {
        let all = self.get_all_living();
        _ = ui_weak.upgrade_in_event_loop(move |h| {
            let cards: Vec<LiveCardInfo> = all
                .into_iter()
                .filter_map(|a| match a.to_card_info() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        error!("anchor info to card info err {}", e);
                        None
                    }
                })
                .collect();
            let mut card_rows = vec![];
            cards.chunks(3).for_each(|vs| {
                let mut s = vec![];
                for v in vs {
                    s.push(v.to_owned());
                }
                card_rows.push(VecModel::from_slice(&s));
            });

            let m = h.global::<g_model>();
            m.set_card_rows(VecModel::from_slice(&card_rows));
        });
    }
}
