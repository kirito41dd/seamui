

use iced::{
    overlay,
    widget::{self, pick_list, row, text_input},
    Element,
};
use iced_lazy::Component;

use strum::IntoEnumIterator;

use crate::model::{self, AnchorInfo};

pub struct AnchorInput<Message> {
    pick_list_item: Vec<String>,
    pick_list_selected: String,
    input: String,
    on_submit: Option<Box<dyn Fn(AnchorInfo) -> Message>>,
}

#[derive(Debug, Clone)]
pub enum AnchorInputMessage {
    Selected(String),
    OnInput(String),
    OnSubmit,
}

impl<Message> AnchorInput<Message> {
    pub fn new() -> Self {
        let plat = model::Platform::iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>();
        let selected = plat[0].clone();
        Self {
            pick_list_item: plat,
            pick_list_selected: selected,
            input: String::new(),
            on_submit: None,
        }
    }

    pub fn on_submit<F: 'static + Fn(AnchorInfo) -> Message>(mut self, f: F) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }
}

impl<Message, Renderer> Component<Message, Renderer> for AnchorInput<Message>
where
    Renderer: iced_native::text::Renderer + 'static,
    Renderer::Theme: widget::pick_list::StyleSheet
        + widget::scrollable::StyleSheet
        + overlay::menu::StyleSheet
        + widget::container::StyleSheet
        + widget::text_input::StyleSheet,
    <Renderer::Theme as overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as widget::pick_list::StyleSheet>::Style>,
{
    type State = ();

    type Event = AnchorInputMessage;

    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<Message> {
        let r = match event {
            AnchorInputMessage::Selected(s) => {
                self.pick_list_selected = s;
                None
            }
            AnchorInputMessage::OnInput(s) => {
                self.input = s;
                None
            }
            AnchorInputMessage::OnSubmit => {
                println!("{}", self.input);
                if self.input.is_empty() {
                    return None;
                }
                let r = if let Some(cb) = &self.on_submit {
                    Some(cb(AnchorInfo {
                        name: self.input.clone(),
                        platform: Some(self.pick_list_selected.as_str().into()),
                        room_id: self.input.clone(),
                        show_type: None,
                    }))
                } else {
                    None
                };

                self.input.clear();
                r
            }
        };

        r
    }

    fn view(&self, _state: &Self::State) -> Element<'_, Self::Event, Renderer> {
        let pick = pick_list(
            &self.pick_list_item,
            Some(self.pick_list_selected.clone()),
            AnchorInputMessage::Selected,
        );

        let input = text_input("room id", &self.input)
            .on_input(AnchorInputMessage::OnInput)
            .on_submit(AnchorInputMessage::OnSubmit);

        row!(pick, input).spacing(15).into()
    }
}

impl<'a, Message, Renderer> From<AnchorInput<Message>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_native::text::Renderer + 'static,
    Renderer::Theme: widget::pick_list::StyleSheet
        + widget::scrollable::StyleSheet
        + overlay::menu::StyleSheet
        + widget::container::StyleSheet
        + widget::text_input::StyleSheet,
    <Renderer::Theme as overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as widget::pick_list::StyleSheet>::Style>,
{
    fn from(numeric_input: AnchorInput<Message>) -> Self {
        iced_lazy::component(numeric_input)
    }
}
