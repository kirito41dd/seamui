use std::{borrow::Borrow, cell::RefMut};

use iced::{
    theme,
    widget::{button, pick_list, row, text, text_input},
    Element, Length,
};

use iced_lazy::Component;

use strum::IntoEnumIterator;

use super::model::*;

use super::uitl::AWESOME;

pub struct AnchorInput<'a, Message> {
    state: RefMut<'a, AnchorInputState>,
    on_submit: Option<Box<dyn Fn(AnchorInfo) -> Message>>,
    on_flush: Option<Box<dyn Fn() -> Message>>,
}

#[derive(Default)]
pub struct AnchorInputState {
    pick_list_item: Vec<String>,
    pick_list_selected: String,
    input: String,
}

#[derive(Debug, Clone)]
pub enum AnchorInputMessage {
    Selected(String),
    OnInput(String),
    OnSubmit,
    OnFlush,
}

impl<'a, Message> AnchorInput<'a, Message> {
    pub fn new(mut selected: RefMut<'a, AnchorInputState>) -> Self {
        let plat = Platform::iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>();

        if selected.borrow().pick_list_selected.is_empty() {
            selected.pick_list_selected = plat[0].clone();
            selected.pick_list_item = plat;
        }

        Self {
            state: selected,
            on_submit: None,
            on_flush: None,
        }
    }

    pub fn on_submit<F: 'static + Fn(AnchorInfo) -> Message>(mut self, f: F) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }
    pub fn on_flush<F: 'static + Fn() -> Message>(mut self, f: F) -> Self {
        self.on_flush = Some(Box::new(f));
        self
    }
}

impl<'a, Message> Component<Message, iced::Renderer> for AnchorInput<'a, Message>
{
    type State = ();

    type Event = AnchorInputMessage;

    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<Message> {
        let r = match event {
            AnchorInputMessage::Selected(s) => {
                self.state.pick_list_selected = s;
                None
            }
            AnchorInputMessage::OnInput(s) => {
                self.state.input = s;
                None
            }
            AnchorInputMessage::OnSubmit => {
                log::info!("submit {}", self.state.input);
                if self.state.input.is_empty() {
                    return None;
                }
                let r = if let Some(cb) = &self.on_submit {
                    Some(cb(AnchorInfo {
                        name: self.state.input.clone(),
                        platform: Some(self.state.pick_list_selected.as_str().into()),
                        room_id: self.state.input.clone(),
                        show_type: None,
                    }))
                } else {
                    None
                };

                self.state.input.clear();
                r
            }
            AnchorInputMessage::OnFlush => {
                if let Some(cb) = &self.on_flush {
                    return Some(cb());
                }
                None
            }
        };

        r
    }

    fn view(&self, _state: &Self::State) -> Element<'_, Self::Event, iced::Renderer> {
        let pick = pick_list(
            &self.state.pick_list_item,
            Some(self.state.pick_list_selected.to_string()),
            AnchorInputMessage::Selected,
        );

        let input = text_input("room id", &self.state.input)
            .on_input(AnchorInputMessage::OnInput)
            .on_submit(AnchorInputMessage::OnSubmit);

        let flush = button(text("\u{f021}").font(AWESOME))
            .style(theme::Button::Secondary.into())
            .on_press(AnchorInputMessage::OnFlush);

        row!(pick, input.width(Length::Fill), flush).spacing(10).align_items(iced::Alignment::Center).into()
    }
}

impl<'a, 'b: 'a, Message> From<AnchorInput<'b, Message>>
    for Element<'a, Message, iced::Renderer>
where
    Message: 'a,
{
    fn from(numeric_input: AnchorInput<'b, Message>) -> Self {
        iced_lazy::component(numeric_input)
    }
}
