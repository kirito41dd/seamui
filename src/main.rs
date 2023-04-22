use iced::{window, Application, Settings};

mod app;

fn main() -> iced::Result {
    env_logger::init();
    app::SeamUI::run(Settings {
        window: window::Settings {
            size: (500, 800),

            ..window::Settings::default()
        },
        default_font: Some(include_bytes!("../static/SIMHEI.TTF")),
        antialiasing: true,
        ..Settings::default()
    })
}
