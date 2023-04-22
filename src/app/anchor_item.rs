use iced::{
    theme,
    widget::{self, button, row, text, text_input},
    Element, Font, Length,
};
use iced_lazy::Component;
use iced_native::column;

use super::{
    model::{self, ShowType},
    uitl::AWESOME,
};

pub struct AnchorItem<'a, Message> {
    info: &'a model::AnchorInfo,
    on_play: Option<Box<dyn Fn(model::Node) -> Message>>,
    on_update: Option<Box<dyn Fn(AnchorItemUpdateType) -> Message>>,
    show_edit: bool,
    name_editor: String,
}

#[derive(Debug, Clone)]
pub enum AnchorItemMessage {
    OnPlay,
    OnEdit,
    CloseEdit,
    OnDel,
    OnEditSubmit,
    OnEditInput(String),
}

#[derive(Debug, Clone)]
pub enum AnchorItemUpdateType {
    Del,
    Update(model::AnchorInfo),
}

impl<'a, Message> AnchorItem<'a, Message> {
    pub fn new(info: &'a model::AnchorInfo) -> Self {
        Self {
            info,
            on_play: None,
            on_update: None,
            show_edit: false,
            name_editor: "".into(),
        }
    }
    pub fn on_play<F: 'static + Fn(model::Node) -> Message>(mut self, f: F) -> Self {
        self.on_play = Some(Box::new(f));
        self
    }
    pub fn on_update<F: 'static + Fn(AnchorItemUpdateType) -> Message>(mut self, f: F) -> Self {
        self.on_update = Some(Box::new(f));
        self
    }
}
impl<'a, Message, Renderer> Component<Message, Renderer> for AnchorItem<'a, Message>
where
    Renderer: iced_native::text::Renderer + 'static,
    Renderer::Theme:
        widget::text::StyleSheet + widget::button::StyleSheet + widget::text_input::StyleSheet,
    <Renderer as iced_native::text::Renderer>::Font: From<Font>,
    <Renderer::Theme as widget::button::StyleSheet>::Style: From<theme::Button>,
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
            AnchorItemMessage::OnEdit => {
                self.name_editor = self.info.name.clone();
                self.show_edit = true;
                None
            }
            AnchorItemMessage::CloseEdit => {
                self.show_edit = false;
                None
            }
            AnchorItemMessage::OnDel => {
                self.show_edit = false;
                if let Some(cb) = self.on_update.as_ref() {
                    return cb(AnchorItemUpdateType::Del).into();
                }
                None
            }
            AnchorItemMessage::OnEditSubmit => {
                self.show_edit = false;
                if self.name_editor == self.info.name {
                    return None;
                }
                if let Some(cb) = self.on_update.as_ref() {
                    let mut new_item = self.info.clone();
                    new_item.name = self.name_editor.clone();
                    return cb(AnchorItemUpdateType::Update(new_item)).into();
                }
                None
            }
            AnchorItemMessage::OnEditInput(s) => {
                self.name_editor = s;
                None
            }
        }
    }

    fn view(&self, _state: &Self::State) -> iced_native::Element<'_, Self::Event, Renderer> {
        if !self.show_edit {
            let name = format!(
                "{}:{}",
                self.info.platform.as_ref().unwrap().as_ui_text(),
                self.info.name
            );
            let room = text(name);

            let mut play = button(text("观看"));
            let mut title = text("");

            let status = if let Some(ShowType::On(s)) = &self.info.show_type {
                play = play.on_press(AnchorItemMessage::OnPlay);
                title = text(&s.title);
                text("直播中")
            } else {
                text("未开播")
            };

            let edit = button(text('\u{f044}').font(AWESOME))
                .padding([0, 0, 0, 10])
                .style(theme::Button::Text.into())
                .on_press(AnchorItemMessage::OnEdit);

            row!(
                column!(row!(room, status), row!(title))
                    .spacing(3)
                    .width(Length::Fill),
                row!(play, edit).padding([0, 20, 0, 0])
            )
            .spacing(10)
            .into()
        } else {
            let edit_name = text_input("", &self.name_editor)
                .on_input(AnchorItemMessage::OnEditInput)
                .on_submit(AnchorItemMessage::OnEditSubmit);
            let del = button(text("\u{F1F8}").font(AWESOME))
                .style(theme::Button::Destructive.into())
                .on_press(AnchorItemMessage::OnDel);
            let close = button(text("\u{f00d}").font(AWESOME))
                .style(theme::Button::Secondary.into())
                .on_press(AnchorItemMessage::CloseEdit);
            row!(
                column!(row!(text("name:"), edit_name).spacing(10)).width(Length::Fill),
                row!(del, close).spacing(10).padding([0, 20, 0, 0])
            )
            .spacing(10)
            .into()
        }
    }
}

impl<'a, 'b: 'a, Message, Renderer> From<AnchorItem<'b, Message>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_native::text::Renderer + 'static,
    Renderer::Theme:
        widget::text::StyleSheet + widget::button::StyleSheet + widget::text_input::StyleSheet,
    <Renderer as iced_native::text::Renderer>::Font: From<Font>,
    <Renderer::Theme as widget::button::StyleSheet>::Style: From<theme::Button>,
{
    fn from(numeric_input: AnchorItem<'b, Message>) -> Self {
        iced_lazy::component(numeric_input)
    }
}
