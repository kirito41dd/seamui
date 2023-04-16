use std::{path::PathBuf};

use serde::{Deserialize, Serialize};
use tokio::process;

use crate::model::{self, AnchorInfo};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SavedState {
    pub anchors: Vec<AnchorInfo>,
}

impl SavedState {
    pub fn path() -> std::path::PathBuf {
        let mut p: PathBuf = directories_next::ProjectDirs::from("", "", "seamui")
            .expect("cant find path")
            .data_dir()
            .into();
        p.push("seamui.json");
        p
    }

    pub async fn load() -> anyhow::Result<SavedState> {
        let data = tokio::fs::read(Self::path()).await?;

        let s: Self = match serde_json::from_slice(&data) {
            Ok(v) => v,
            Err(e) => {
                println!("err {}", e);
                Self::default()
            }
        };

        Ok(s)
    }

    pub async fn save(self) -> anyhow::Result<()> {
        let data = serde_json::to_string_pretty(&self)?;

        let path = Self::path();
        tokio::fs::create_dir_all(path.parent().expect("get dir")).await?;
        tokio::fs::File::create(&path).await?;
        tokio::fs::write(path, data).await?;
        Ok(())
    }
}

pub struct PlayState {}

impl PlayState {
    pub async fn play(node: model::Node) -> anyhow::Result<()> {
        let _output = process::Command::new("mpv").arg(node.url).output().await?;
        Ok(())
    }
}
