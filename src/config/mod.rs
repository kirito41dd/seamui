use std::{default, path::Path};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::models::AnchorInfo;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AppConfig {
    pub player_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            player_path: "mpv".into(),
        }
    }
}

pub fn config_path() -> std::path::PathBuf {
    directories_next::ProjectDirs::from("", "", "seamui2")
        .expect("cant find path")
        .data_dir()
        .join("seamui.json")
}

pub fn anchor_path() -> std::path::PathBuf {
    directories_next::ProjectDirs::from("", "", "seamui2")
        .expect("cant find path")
        .data_dir()
        .join("anchor.json")
}

pub fn cache_path() -> std::path::PathBuf {
    directories_next::ProjectDirs::from("", "", "seamui2")
        .expect("cant find path")
        .cache_dir()
        .into()
}

pub async fn load_config() -> anyhow::Result<AppConfig> {
    let data = tokio::fs::read(config_path()).await?;
    Ok(serde_json::from_slice(&data)?)
}

pub async fn load_anchor() -> anyhow::Result<Vec<AnchorInfo>> {
    let data = tokio::fs::read(anchor_path()).await?;
    Ok(serde_json::from_slice(&data)?)
}

pub async fn save_config(config: &AppConfig) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(config_path().parent().ok_or(anyhow!("get parent"))?).await?;
    Ok(tokio::fs::write(config_path(), serde_json::to_string_pretty(config)?).await?)
}

pub async fn save_anchor(anchors: &[AnchorInfo]) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(anchor_path().parent().ok_or(anyhow!("get parent"))?).await?;
    Ok(tokio::fs::write(anchor_path(), serde_json::to_string_pretty(anchors)?).await?)
}
