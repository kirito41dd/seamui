use std::sync::Arc;

use seam_core::{error::SeamError, live::Format};
use tokio::sync::{mpsc, Mutex, Semaphore};

use crate::app::model::{self, ShowType};

use super::{
    model::{AnchorInfo, Node, SeamInfo},
    uitl::AppConfig,
};

pub struct SeamServer {
    result_sender: Arc<Mutex<mpsc::UnboundedSender<AnchorInfo>>>,
    task_receiver: mpsc::UnboundedReceiver<(AnchorInfo, AppConfig)>,
}

impl SeamServer {
    pub fn new(
        result_sender: mpsc::UnboundedSender<AnchorInfo>,
        task_receiver: mpsc::UnboundedReceiver<(AnchorInfo, AppConfig)>,
    ) -> Self {
        SeamServer {
            result_sender: Arc::new(Mutex::new(result_sender)),
            task_receiver,
        }
    }

    pub async fn run(mut self) {
        let semaphore = Arc::new(Semaphore::new(5));

        while let Some((mut info, _cfg)) = self.task_receiver.recv().await {
            let sender = self.result_sender.clone();
            let guard = semaphore.clone().acquire_owned().await.unwrap();
            tokio::spawn(async move {
                let _g = guard;
                let cli = info.platform.unwrap().get_live();
                let output = cli.get(&info.room_id).await;

                match output {
                    Ok(out) => {
                        let nodes: Vec<_> = out
                            .urls
                            .into_iter()
                            .map(|u| -> Node {
                                let str = match u.format {
                                    Format::Flv => "flv".to_string(),
                                    Format::M3U => "m3u".to_string(),
                                    Format::Rtmp => "rtmp".to_owned(),
                                    Format::Other(s) => s,
                                };
                                Node {
                                    format: str,
                                    url: u.url,
                                }
                            })
                            .collect();
                        let seam_info = SeamInfo {
                            title: out.title,
                            nodes: Some(nodes),
                        };
                        info.show_type = Some(ShowType::On(seam_info));
                        sender.lock().await.send(info).expect("send err");
                    }
                    Err(e) => match e {
                        SeamError::None => {
                            log::info!("seam query result off");
                            info.show_type = Some(model::ShowType::Off);
                            sender.lock().await.send(info).expect("send err");
                        }
                        _ => {
                            log::error!(
                                "seam query {:?} {} err {:?}",
                                info.platform,
                                info.room_id,
                                e
                            );
                        }
                    },
                }
            });
        }
    }
}
