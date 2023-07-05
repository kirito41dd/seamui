use iced::{
    alignment::{Horizontal, Vertical},
    widget::{column, container, scrollable},
    Length, Renderer,
};

use super::{
    anchor_input::AnchorInput, anchor_item::AnchorItem, cfg_panel::CfgPanel, Message, SeamUI,
};

impl SeamUI {
    pub fn main_page_view(
        &self,
    ) -> iced::Element<
        '_,
        <SeamUI as iced::Application>::Message,
        iced::Renderer<<SeamUI as iced::Application>::Theme>,
    > {
        let anchor_input = AnchorInput::new(self.anchor_input_state.borrow_mut())
            .on_submit(Message::SubmitAnchor)
            .on_flush(|| Message::OnFlush)
            .on_setting(|| Message::OnSetting);

        let es: Vec<AnchorItem<Message>> = self
            .anchor_list
            .iter()
            .enumerate()
            .map(|(i, item)| -> AnchorItem<Message> {
                AnchorItem::new(item)
                    .on_play(move |v| Message::OnPlay(i, v))
                    .on_update(move |v| Message::OnItemUpdate(i, v))
            })
            .collect();
        let es: Vec<iced_native::Element<Message, Renderer>> =
            es.into_iter().map(|e| e.into()).collect();

        let c = column(es).align_items(iced::Alignment::Start).spacing(15);

        let content = iced_native::column!(
            anchor_input,
            scrollable(container(c).width(Length::Fill).padding([0, 6, 0, 6]))
        )
        .align_items(iced::Alignment::Center)
        .padding(10)
        .spacing(20);

        container(content)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
            .into()
    }
    pub fn setting_view(
        &self,
    ) -> iced::Element<
        '_,
        <SeamUI as iced::Application>::Message,
        iced::Renderer<<SeamUI as iced::Application>::Theme>,
    > {
        let panel = CfgPanel::new(&self.config).on_update(Message::OnSettingUpdate);
        let c = column!(panel)
            .align_items(iced::Alignment::Center)
            .padding(10)
            .spacing(20);
        let content = scrollable(container(c).width(Length::Fill).padding([0, 6, 0, 6]));
        container(content)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
            .into()
    }
}
