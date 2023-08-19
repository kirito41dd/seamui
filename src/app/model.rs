use seam_core::live::Live;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AnchorInfo {
    pub name: String,
    pub platform: Option<Platform>,
    pub room_id: String,
    #[serde(skip)]
    pub show_type: Option<ShowType>,
}

#[derive(Debug, strum::EnumIter, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub enum Platform {
    BiliBili,
    DouYu,
    DouYin,
    HuYa,
    KuaiShou,
    CC,
    HuaJiao,
    Now,
    Afreeca,
}

impl From<&str> for Platform {
    fn from(value: &str) -> Self {
        match value {
            "BiliBili" => Self::BiliBili,
            "DouYu" => Self::DouYu,
            "DouYin" => Self::DouYin,
            "HuYa" => Self::HuYa,
            "KuaiShou" => Self::KuaiShou,
            "CC" => Self::CC,
            "HuaJiao" => Self::HuaJiao,
            "Now" => Self::Now,
            "Afreeca" => Self::Afreeca,
            _ => Self::BiliBili,
        }
    }
}

impl Platform {
    pub fn as_seam_arg(&self) -> &'static str {
        match self {
            Platform::BiliBili => "bili",
            Platform::DouYu => "douyu",
            Platform::DouYin => "douyin",
            Platform::HuYa => "huya",
            Platform::KuaiShou => "kuaishou",
            Platform::CC => "cc",
            Platform::HuaJiao => "huajiao",
            Platform::Now => "now",
            Platform::Afreeca => "afreeca",
        }
    }
    pub fn as_ui_text(&self) -> &'static str {
        match self {
            Platform::BiliBili => "B站",
            Platform::DouYu => "斗鱼",
            Platform::DouYin => "抖音",
            Platform::HuYa => "虎牙",
            Platform::KuaiShou => "快手",
            Platform::CC => "CC",
            Platform::HuaJiao => "花椒",
            Platform::Now => "Now",
            Platform::Afreeca => "Afreeca",
        }
    }
    pub async fn get_live(&self, room_id: &str) -> seam_core::error::Result<seam_core::live::Node> {
        match self {
            Platform::BiliBili => seam_core::live::bili::Client {}.get(room_id, None).await,
            Platform::DouYu => seam_core::live::douyu::Client {}.get(room_id, None).await,
            Platform::DouYin => seam_core::live::douyin::Client {}.get(room_id, None).await,
            Platform::HuYa => seam_core::live::huya::Client {}.get(room_id, None).await,
            Platform::KuaiShou => seam_core::live::ks::Client {}.get(room_id, None).await,
            Platform::CC => seam_core::live::cc::Client {}.get(room_id, None).await,
            Platform::HuaJiao => {
                seam_core::live::huajiao::Client {}
                    .get(room_id, None)
                    .await
            }
            Platform::Now => seam_core::live::now::Client {}.get(room_id, None).await,
            Platform::Afreeca => {
                seam_core::live::afreeca::Client {}
                    .get(room_id, None)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ShowType {
    // 开播
    On(SeamInfo),
    // 未开播
    Off,
    // 错误
    Error(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SeamInfo {
    #[serde(default)]
    pub title: String,
    pub nodes: Option<Vec<Node>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Node {
    pub format: String,
    pub url: String,
}

#[cfg(test)]
mod tests {

    use super::SeamInfo;

    #[test]
    fn test_encode_decode() {
        let info = serde_json::from_str::<SeamInfo>(
            r#"
        {
            "title": "ttttt",
            "nodes": [
              {
                "format": "flv",
                "url": "ddddddd"
              }
            ]
        }"#,
        )
        .unwrap();
        println!("{:?}", info);
        println!("{}", serde_json::to_string(&info).unwrap());
    }
}
