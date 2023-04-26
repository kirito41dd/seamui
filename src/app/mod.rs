use std::{cell::RefCell, ops::IndexMut};

use log::{debug, info};
use tokio::sync::mpsc;

use iced::{
    Application, Command, Subscription,
};

use crate::app::uitl::PlayState;

use self::{
    anchor_input::{AnchorInputState},
    anchor_item::{AnchorItemUpdateType},
    model::*,
    server::SeamServer,
    uitl::{AppConfig, SavedState},
};

mod anchor_input;
mod anchor_item;
mod cfg_panel;
mod model;
mod pages;
mod server;
mod uitl;

pub struct SeamUI {
    loaded: bool,
    anchor_list: Vec<AnchorInfo>,
    anchor_input_state: RefCell<AnchorInputState>,
    task_sender: mpsc::UnboundedSender<(AnchorInfo, AppConfig)>,
    result_receiver: RefCell<Option<mpsc::UnboundedReceiver<AnchorInfo>>>,
    config: AppConfig,
    show_setting: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(SavedState),
    Saved,
    SubmitAnchor(AnchorInfo),
    OnPlay(usize, model::Node),
    OnItemUpdate(usize, AnchorItemUpdateType),
    OnFlush,
    OnSetting,
    OnSettingUpdate(Option<AppConfig>),
    TaskResult(AnchorInfo),
    Ignore,
}

impl Application for SeamUI {
    type Executor = iced::executor::Default;

    type Message = Message;

    type Theme = iced::Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let (result_sender, result_receiver) = mpsc::unbounded_channel::<AnchorInfo>();
        let (task_sender, task_receiver) = mpsc::unbounded_channel::<(AnchorInfo, AppConfig)>();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { SeamServer::new(result_sender, task_receiver).run().await })
        });

        (
            SeamUI {
                loaded: false,
                anchor_input_state: RefCell::new(AnchorInputState::default()),
                anchor_list: vec![],
                task_sender,
                result_receiver: RefCell::new(Some(result_receiver)),
                config: AppConfig::default(),
                show_setting: false,
            },
            Command::perform(SavedState::load(), |r| {
                info!("load is ok {:?}", r.is_ok());
                Message::Loaded(r.expect("load"))
            }),
        )
    }

    fn title(&self) -> String {
        "seamui".into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Loaded(s) => {
                info!("load anchors len={}, cfg:{:?}", s.anchors.len(), s.config);
                self.anchor_list = s.anchors;
                self.config = s.config;
                self.loaded = true;
                self.anchor_list.iter().for_each(|v| {
                    self.task_sender
                        .send((v.clone(), self.config.clone()))
                        .expect("send err");
                });

                Command::none()
            }

            Message::SubmitAnchor(anchor) => {
                self.anchor_list.push(anchor.clone());
                self.task_sender
                    .send((anchor, self.config.clone()))
                    .expect("send err");
                let save = Command::perform(
                    SavedState {
                        anchors: self.anchor_list.clone(),
                        config: self.config.clone(),
                    }
                    .save(),
                    |v| {
                        info!("saved {:?}", v);
                        Message::Saved
                    },
                );
                Command::batch(vec![Command::none(), save])
            }

            Message::OnPlay(i, node) => {
                info!("play idx:{} {:?} {:?}", i, node, self.config);
                Command::perform(PlayState::play(node, self.config.clone()), move |v| {
                    info!("play idx:{} {:?}", i, v);
                    Message::Ignore
                })
            }
            Message::OnItemUpdate(i, typ) => {
                debug!("OnItemUpdate {} {:?}", i, typ);
                match typ {
                    AnchorItemUpdateType::Del => {
                        self.anchor_list.remove(i);
                    }
                    AnchorItemUpdateType::Update(n) => {
                        *self.anchor_list.index_mut(i) = n;
                    }
                }
                Command::perform(
                    SavedState {
                        anchors: self.anchor_list.clone(),
                        config: self.config.clone(),
                    }
                    .save(),
                    |v| {
                        info!("saved due to OnItemUpdate: {:?}", v);
                        Message::Saved
                    },
                )
            }

            Message::TaskResult(info) => {
                self.anchor_list
                    .iter_mut()
                    .filter(|m| {
                        if m.platform == info.platform && m.room_id == info.room_id {
                            return true;
                        }
                        false
                    })
                    .for_each(|v| {
                        v.show_type = info.show_type.clone();
                    });
                Command::none()
            }
            Message::OnFlush => {
                self.anchor_list.iter().for_each(|v| {
                    self.task_sender
                        .send((v.clone(), self.config.clone()))
                        .expect("send err");
                });
                Command::none()
            }
            Message::OnSetting => {
                self.show_setting = true;
                Command::none()
            }
            Message::OnSettingUpdate(s) => {
                self.show_setting = false;
                if let Some(setting) = s {
                    self.config = setting;
                    log::info!("update setting {:?}", self.config);
                    return Command::perform(
                        SavedState {
                            anchors: self.anchor_list.clone(),
                            config: self.config.clone(),
                        }
                        .save(),
                        |v| {
                            info!("saved due to OnSettingUpdate: {:?}", v);
                            Message::Saved
                        },
                    );
                }
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        if self.show_setting {
            self.setting_view()
        } else {
            self.main_page_view()
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let server = iced::subscription::unfold(
            "seam server",
            self.result_receiver.take(),
            move |mut r| async {
                let result = r.as_mut().expect("get r").recv().await.expect("rev r");
                (Message::TaskResult(result), r)
            },
        );

        Subscription::batch([server])
    }
}
