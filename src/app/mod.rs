use std::cell::RefCell;

use tokio::sync::mpsc;

use iced::{
    widget::{column, container, scrollable},
    Application, Command, Length, Renderer, Subscription,
};

use strum::IntoEnumIterator;

use crate::{
    app::uitl::PlayState,
    model::{self, AnchorInfo},
};

use self::{
    anchor_input::AnchorInput, anchor_item::AnchorItem, server::SeamServer, uitl::SavedState,
};

mod anchor_input;
mod anchor_item;
mod server;
mod uitl;

pub struct SeamUI {
    loaded: bool,
    anchor_list: Vec<AnchorInfo>,
    anchor_input: AnchorInput<Message>,
    sub: bool,
    task_sender: mpsc::UnboundedSender<AnchorInfo>,
    result_receiver: RefCell<Option<mpsc::UnboundedReceiver<AnchorInfo>>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(SavedState),
    Saved,
    SubmitAnchor(AnchorInfo),
    OnPlay(usize, model::Node),
    Subs,
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
        let (task_sender, task_receiver) = mpsc::unbounded_channel::<AnchorInfo>();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { SeamServer::new(result_sender, task_receiver).run().await })
        });

        (
            SeamUI {
                loaded: false,
                anchor_input: AnchorInput::new(),
                anchor_list: vec![],
                sub: false,
                task_sender: task_sender,
                result_receiver: RefCell::new(Some(result_receiver)),
            },
            Command::perform(SavedState::load(), |r| {
                println!("load is ok {:?}", r.is_ok());
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
                println!("load anchors len={}", s.anchors.len());
                self.anchor_list = s.anchors;
                self.loaded = true;
                self.task_sender
                    .send(self.anchor_list[0].clone())
                    .expect("send err");
                Command::none()
            }

            Message::SubmitAnchor(anchor) => {
                println!("{:?}, path {:?}", anchor, SavedState::path());
                self.anchor_list.push(anchor);

                let save = Command::perform(
                    SavedState {
                        anchors: self.anchor_list.clone(),
                    }
                    .save(),
                    |v| {
                        println!("saved? {:?}", v);
                        Message::Saved
                    },
                );
                Command::batch(vec![Command::none(), save])
            }

            Message::OnPlay(i, node) => {
                println!("{} {:?}", i, node);
                Command::perform(PlayState::play(node), move |v| {
                    println!("play {} {:?}", i, v);
                    Message::Ignore
                })
            }
            Message::Subs => {
                println!("get subs");
                self.sub = true;
                Command::none()
            }

            Message::TaskResult(info) => {
                println!("recv task {:?}", info);
                self.anchor_list
                    .iter_mut()
                    .filter(|m| {
                        if m.platform == info.platform && m.room_id == info.room_id {
                            return true;
                        }
                        return false;
                    })
                    .for_each(|v| {
                        v.show_type = info.show_type.clone();
                    });
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let anchor_input = AnchorInput::new().on_submit(Message::SubmitAnchor);

        let es: Vec<AnchorItem<Message>> = self
            .anchor_list
            .iter()
            .enumerate()
            .map(|(i, item)| -> AnchorItem<Message> {
                AnchorItem::new(item).on_play(move |v| Message::OnPlay(i, v))
            })
            .collect();
        let es: Vec<iced_native::Element<Message, Renderer>> =
            es.into_iter().map(|e| e.into()).collect();

        let c = column(es).spacing(10);

        let content = iced_native::column!(
            anchor_input,
            scrollable(container(c).width(Length::Fill).center_x())
        )
        .padding(20)
        .spacing(20);

        content.into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        println!("subscription");
        let server = iced::subscription::unfold(
            "seam server",
            self.result_receiver.take(),
            move |mut r| async {
                let result = r.as_mut().expect("get r").recv().await.expect("rev r");
                (Message::TaskResult(result), r)
            },
        );

        return Subscription::batch([server]);
    }
}
