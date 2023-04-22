use std::sync::Arc;

use tokio::sync::{mpsc, Mutex, Semaphore};

use crate::app::model::{self, ShowType};

use super::model::{AnchorInfo, SeamInfo};

pub struct SeamServer {
    result_sender: Arc<Mutex<mpsc::UnboundedSender<AnchorInfo>>>,
    task_receiver: mpsc::UnboundedReceiver<AnchorInfo>,
}

impl SeamServer {
    pub fn new(
        result_sender: mpsc::UnboundedSender<AnchorInfo>,
        task_receiver: mpsc::UnboundedReceiver<AnchorInfo>,
    ) -> Self {
        SeamServer {
            result_sender: Arc::new(Mutex::new(result_sender)),
            task_receiver,
        }
    }

    pub async fn run(mut self) {
        let semaphore = Arc::new(Semaphore::new(5));
        loop {
            if let Some(mut info) = self.task_receiver.recv().await {
                let sender = self.result_sender.clone();
                let guard = semaphore.clone().acquire_owned().await.unwrap();
                tokio::spawn(async move {
                    let _g = guard;
                    let output = tokio::process::Command::new("seam")
                        .args([
                            info.platform.expect("platform").as_seam_arg(),
                            info.room_id.as_str(),
                        ])
                        .output()
                        .await;

                    if let Ok(out) = output {
                        if let Ok(seam_info) = serde_json::from_slice::<SeamInfo>(&out.stdout) {
                            log::info!("seam query result {:?}", seam_info.title);
                            info.show_type = Some(ShowType::On(seam_info))
                        } else {
                            log::info!("seam query result off");
                            info.show_type = Some(model::ShowType::Off)
                        }

                        sender.lock().await.send(info).expect("send err");
                    } else {
                        log::error!(
                            "seam query {:?} {} result {:?}",
                            info.platform,
                            info.room_id,
                            output
                        );
                    }
                });
            } else {
                break;
            }
        }
    }
}
