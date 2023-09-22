use std::rc::Rc;

use log::{error, info};
use slint::{ModelRc, SharedString, VecModel};
use tokio::process;

pub fn vec_string_to_slint_model(ss: &[String]) -> ModelRc<SharedString> {
    let v = VecModel::default();
    for s in ss {
        v.push(s.into());
    }
    ModelRc::from(Rc::new(v))
}

pub async fn play(url: String) -> anyhow::Result<()> {
    info!("start play: {}", url);
    tokio::spawn(async {
        let output = process::Command::new("mpv").arg(url).output().await;
        match output {
            Ok(_) => {}
            Err(e) => {
                error!("play  err: {}", e);
            }
        }
    });
    Ok(())
}
