use iced::{
    theme,
    widget::{button, row, text, text_input},
    Element, Length,
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
    OnLiveLineSwitch(i32),
    None(String),
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
impl<'a, Message> Component<Message, iced::Renderer> for AnchorItem<'a, Message> {
    type State = AnchorItemState;

    type Event = AnchorItemMessage;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            AnchorItemMessage::OnPlay => {
                if let Some(cb) = self.on_play.as_ref() {
                    if let Some(ShowType::On(seam_info)) = &self.info.show_type {
                        if let Some(nodes) = &seam_info.nodes {
                            if let Some(n) = nodes.get(state.live_line) {
                                return Some(cb(n.clone()));
                            }
                            return None;
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
            AnchorItemMessage::None(_) => None,
            AnchorItemMessage::OnLiveLineSwitch(i) => {
                if let Some(ShowType::On(s)) = &self.info.show_type {
                    if let Some(n) = &s.nodes {
                        if !n.is_empty() {
                            if i > 0 {
                                state.live_line = (state.live_line + 1) % n.len();
                            } else {
                                state.live_line = (state.live_line + n.len() - 1) % n.len();
                            }
                        }
                    }
                }
                None
            }
        }
    }

    fn view(&self, state: &Self::State) -> iced_native::Element<'_, Self::Event, iced::Renderer> {
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
                text("直播中:")
            } else {
                text("未开播")
            };

            let edit = button(text('\u{f304}').font(AWESOME).size(17))
                .style(theme::Button::Text)
                .on_press(AnchorItemMessage::OnEdit);

            column!(
                row!(room, row!().width(Length::Fill), play, edit)
                    .spacing(5)
                    .align_items(iced::Alignment::Center),
                row!(status, title)
                    .spacing(5)
                    .align_items(iced::Alignment::Center)
            )
            .spacing(3)
            .align_items(iced::Alignment::Start)
            .into()
        } else {
            let edit_name = text_input("", &self.name_editor)
                .on_input(AnchorItemMessage::OnEditInput)
                .on_submit(AnchorItemMessage::OnEditSubmit);
            let del = button(text("\u{f1f8}").font(AWESOME))
                .style(theme::Button::Destructive)
                .on_press(AnchorItemMessage::OnDel);
            let close = button(text("\u{f00d}").font(AWESOME))
                .style(theme::Button::Text)
                .on_press(AnchorItemMessage::CloseEdit);

            let live_line_title = text(format!("线路{}:", state.live_line + 1));
            let mut live_line_format = text("");
            let mut live_line_input = text_input("", "");

            let live_line_switch_pre = button(text("\u{f053}").font(AWESOME))
                .style(theme::Button::Text)
                .on_press(AnchorItemMessage::OnLiveLineSwitch(-1));
            let live_line_switch_next = button(text("\u{f054}").font(AWESOME))
                .style(theme::Button::Text)
                .on_press(AnchorItemMessage::OnLiveLineSwitch(1));
            if let Some(ShowType::On(s)) = &self.info.show_type {
                if let Some(nodes) = &s.nodes {
                    if let Some(node) = nodes.get(state.live_line) {
                        live_line_input =
                            text_input("", &node.url).on_input(AnchorItemMessage::None);
                        live_line_format = text(&node.format);
                    }
                }
            }

            column!(
                row!(text("名称:"), edit_name.width(Length::Fill), del, close)
                    .spacing(5)
                    .align_items(iced::Alignment::Center),
                row!(
                    live_line_title,
                    live_line_input.width(Length::Fill),
                    live_line_format,
                    live_line_switch_pre,
                    live_line_switch_next
                )
                .spacing(5)
                .align_items(iced::Alignment::Center)
            )
            .spacing(3)
            .align_items(iced::Alignment::Start)
            .into()
        }
    }
}

impl<'a, 'b: 'a, Message> From<AnchorItem<'b, Message>> for Element<'a, Message, iced::Renderer>
where
    Message: 'a,
{
    fn from(numeric_input: AnchorItem<'b, Message>) -> Self {
        iced_lazy::component(numeric_input)
    }
}

#[derive(Default, Clone, Debug)]
pub struct AnchorItemState {
    live_line: usize,
}
