use std::{
    sync::Arc,
    thread,
    time::{self, Duration},
};

mod config;
use config::AppConfig;
mod cache;
mod models;
mod server;
mod util;
use log::info;
use once_cell::sync::Lazy;
use server::Server;
use slint::{invoke_from_event_loop, platform, Model, ModelRc, SharedString, VecModel};

slint::include_modules!();

static CFG: Lazy<std::sync::Mutex<AppConfig>> = Lazy::new(|| {
    futures::executor::block_on(async {
        let cfg = match config::load_config().await {
            Ok(v) => v,
            Err(_) => AppConfig::default(),
        };
        std::sync::Mutex::new(cfg)
    })
});

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let ui = AppWindow::new()?;

    let weak_ui: slint::Weak<AppWindow> = ui.as_weak();

    let server = Arc::new(Server::new());

    let g_model = ui.global::<g_model>();
    let server_copy = server.clone();
    g_model.on_follow(move |platform, room| {
        println!("follow: {} {}", platform, room);
        _ = server_copy
            .push_msg(server::Msg::Follow(platform.to_string(), room.to_string()))
            .unwrap();
        println!("send success");
    });
    let server_copy = server.clone();
    g_model.on_want_play(move |card_info: LiveCardInfo| {
        let urls = card_info
            .urls
            .as_any()
            .downcast_ref::<VecModel<SharedString>>()
            .expect("down cast");
        let urls = urls.iter().map(|v| v.clone().to_string()).collect();
        info!("on_want_play: {}", card_info.title);
        _ = server_copy.push_msg(server::Msg::Play(urls));
    });

    let server_copy = server.clone();
    let handle = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        runtime.block_on(async {
            server_copy.start_loop(weak_ui.clone()).await;
            std::process::exit(0);
        });
    });
    ui.show()?;
    ui.run()?;
    server.push_msg(server::Msg::Exit).unwrap();
    _ = handle.join().unwrap();
    // _ = handle.join();
    Ok(())
}
