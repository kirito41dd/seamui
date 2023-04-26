use iced::{
    theme::Button,
    widget::{button, text, text_input},
    Element, Length,
};
use iced_lazy::Component;
use iced_native::row;

use super::uitl::{AppConfig, SavedState, AWESOME};

pub struct CfgPanel<'a, Message> {
    cfg: &'a AppConfig,
    on_update: Option<Box<dyn Fn(Option<AppConfig>) -> Message>>,
}

#[derive(Clone)]
pub enum CfgPanelMessage {
    OnSave,
    OnOff,
    OnInputSeamPath(String),
    OninputPlayerPath(String),
    None,
}

#[derive(Default)]
pub struct CfgPanelState {
    inited: bool,
    cfg: AppConfig,
}

impl<'a, Message> CfgPanel<'a, Message> {
    pub fn new(cfg: &'a AppConfig) -> Self {
        Self {
            cfg,
            on_update: None,
        }
    }
    pub fn on_update<F: 'static + Fn(Option<AppConfig>) -> Message>(mut self, f: F) -> Self {
        self.on_update = Some(Box::new(f));
        self
    }
}

impl<'a, Message> Component<Message, iced::Renderer> for CfgPanel<'a, Message> {
    type State = CfgPanelState;

    type Event = CfgPanelMessage;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        if !state.inited {
            state.inited = true;
            state.cfg = self.cfg.clone();
        }
        match event {
            CfgPanelMessage::OnOff => {
                *state = Default::default();
                if let Some(cb) = self.on_update.as_ref() {
                    return Some(cb(None));
                }
                None
            }
            CfgPanelMessage::OnSave => {
                if let Some(cb) = self.on_update.as_ref() {
                    if state.cfg == *self.cfg {
                        return Some(cb(None));
                    }
                    let msg = Some(cb(Some(state.cfg.clone())));
                    *state = Default::default();
                    return msg;
                }
                None
            }
            CfgPanelMessage::OnInputSeamPath(s) => {
                state.cfg.seam_path = s;
                None
            }
            CfgPanelMessage::OninputPlayerPath(s) => {
                state.cfg.player_path = s;
                None
            }
            CfgPanelMessage::None => None,
        }
    }

    fn view(&self, state: &Self::State) -> iced_native::Element<'_, Self::Event, iced::Renderer> {
        let cfg = if state.inited { &state.cfg } else { self.cfg };

        let title = iced_native::row!(text("设置")
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .width(Length::Fill)
            .size(40));

        let seam_path = text("seam路径:");
        let seam_path_input =
            text_input("", &cfg.seam_path).on_input(CfgPanelMessage::OnInputSeamPath);

        let player_path = text("播放器路径:");
        let player_path_input =
            text_input("", &cfg.player_path).on_input(CfgPanelMessage::OninputPlayerPath);

        let config_path = "配置文件路径:";
        let config_path_input = text_input("", SavedState::path().to_str().unwrap())
            .on_input(|_| CfgPanelMessage::None);

        let github = "项目地址:";
        let github_input = text_input("", "https://github.com/kirito41dd/seamui")
            .on_input(|_| CfgPanelMessage::None);

        let group = "群组:";
        let group_input =
            text_input("", "https://t.me/seam_rust").on_input(|_| CfgPanelMessage::None);

        let ok = button(text("\u{f00c}").font(AWESOME).size(25)).on_press(CfgPanelMessage::OnSave);
        let off = button(text("\u{f00d}").font(AWESOME).size(25))
            .style(Button::Destructive)
            .on_press(CfgPanelMessage::OnOff);
        let buttons = iced_native::row!(
            row!().width(Length::Fill),
            ok,
            off,
            row!().width(Length::Fill)
        )
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .align_items(iced::Alignment::Center);
        iced_native::column!(
            title,
            seam_path,
            seam_path_input,
            player_path,
            player_path_input,
            config_path,
            config_path_input,
            github,
            github_input,
            group,
            group_input,
            buttons,
        )
        .spacing(5)
        .into()
    }
}

impl<'a, 'b: 'a, Message> From<CfgPanel<'b, Message>> for Element<'a, Message, iced::Renderer>
where
    Message: 'a,
{
    fn from(cfg_panel: CfgPanel<'b, Message>) -> Self {
        iced_lazy::component(cfg_panel)
    }
}
