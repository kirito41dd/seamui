use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

use crate::model::{AnchorInfo, SeamInfo};

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
        loop {
            if let Some(mut info) = self.task_receiver.recv().await {
                let sender = self.result_sender.clone();
                tokio::spawn(async move {
                    let output = tokio::process::Command::new("seam")
                        .args(&[
                            info.platform.expect("platform").as_seam_arg(),
                            info.room_id.as_str(),
                        ])
                        .output()
                        .await;

                    if let Ok(out) = output {
                        println!("seam output {:?}", out);
                        if let Ok(seam_info) = serde_json::from_slice::<SeamInfo>(&out.stdout) {
                            info.show_type = Some(crate::model::ShowType::On(seam_info))
                        } else {
                            info.show_type = Some(crate::model::ShowType::Off)
                        }
                        println!("{:?}", &info);
                        sender.lock().await.send(info).expect("send err");
                    }
                });
            } else {
                break;
            }
        }
    }
}
