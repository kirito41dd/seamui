

use iced::{
    widget::{self, button, row, text},
    Element, Length,
};
use iced_lazy::Component;

use crate::model::{self, ShowType};

pub struct AnchorItem<'a, Message> {
    info: &'a model::AnchorInfo,
    on_play: Option<Box<dyn Fn(model::Node) -> Message>>,
}

#[derive(Debug, Clone)]
pub enum AnchorItemMessage {
    OnPlay,
}

impl<'a, Message> AnchorItem<'a, Message> {
    pub fn new(info: &'a model::AnchorInfo) -> Self {
        Self {
            info,
            on_play: None,
        }
    }
    pub fn on_play<F: 'static + Fn(model::Node) -> Message>(mut self, f: F) -> Self {
        self.on_play = Some(Box::new(f));
        self
    }
}
impl<'a, Message, Renderer> Component<Message, Renderer> for AnchorItem<'a, Message>
where
    Renderer: iced_native::text::Renderer + 'static,
    Renderer::Theme: widget::text::StyleSheet + widget::button::StyleSheet,
{
    type State = ();

    type Event = AnchorItemMessage;

    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            AnchorItemMessage::OnPlay => {
                if let Some(cb) = self.on_play.as_ref() {
                    if let Some(ShowType::On(seam_info)) = &self.info.show_type {
                        if let Some(nodes) = &seam_info.nodes {
                            return Some(cb(nodes[0].clone()));
                        }
                    }
                }
                None
            }
        }
    }

    fn view(&self, _state: &Self::State) -> iced_native::Element<'_, Self::Event, Renderer> {
        let room = text(format!(
            "{:?}:{}",
            self.info.platform.as_ref().unwrap(),
            self.info.room_id
        ));

        let status = if self.info.show_type.is_none() {
            text("未开播")
        } else {
            text("直播中")
        };
        let mut play = button("观看");
        if self.info.show_type.is_some() {
            play = play.on_press(AnchorItemMessage::OnPlay);
        }

        row!(
            row!(room, status).width(Length::Fill),
            row!(play).padding([0, 20, 0, 0])
        )
        .spacing(10)
        .into()
    }
}

impl<'a, 'b: 'a, Message, Renderer> From<AnchorItem<'b, Message>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_native::text::Renderer + 'static,
    Renderer::Theme: widget::text::StyleSheet + widget::button::StyleSheet,
{
    fn from(numeric_input: AnchorItem<'b, Message>) -> Self {
        iced_lazy::component(numeric_input)
    }
}
