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
}

impl From<&str> for Platform {
    fn from(value: &str) -> Self {
        match value {
            "BiliBili" => Self::BiliBili,
            "DouYu" => Self::DouYu,
            "DouYin" => Self::DouYin,
            "HuYa" => Self::HuYa,
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
        }
    }
    pub fn as_ui_text(&self) -> &'static str {
        match self {
            Platform::BiliBili => "b站",
            Platform::DouYu => "斗鱼",
            Platform::DouYin => "抖音",
            Platform::HuYa => "虎牙",
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
