use std::{collections::HashMap, path::PathBuf, rc::Rc};

use anyhow::anyhow;
use seam_core::live::Live;
use serde::{Deserialize, Serialize};
use slint::{ModelRc, SharedString, VecModel};

use crate::cache;

type Node = seam_core::live::Node;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AnchorInfo {
    pub rid: String,      // 房间号
    pub title: String,    // 标题
    pub cover: String,    // 封面 会被自动转换到本地路径
    pub anchor: String,   // 主播名
    pub head: String,     // 头像 会被自动转换到本地路径
    pub platform: String, // 平台
    #[serde(skip)]
    pub show_type: Option<ShowType>,
    pub cover_path: Option<PathBuf>,
    pub head_path: Option<PathBuf>,
}

impl AnchorInfo {
    pub async fn get(platform: &str, room_id: &str) -> anyhow::Result<Self> {
        let node = match platform {
            "bilibili" => seam_core::live::bili::Client {}.get(room_id, None).await?,
            "douyu" => {
                let mut header: HashMap<String, String> = HashMap::new();
                header.insert("Cookie".into(), "dy_did=39a3c2108291d877700c141f00001601; acf_did=39a3c2108291d877700c141f00001601; dy_teen_mode=%7B%22uid%22%3A%2216797974%22%2C%22status%22%3A0%2C%22birthday%22%3A%22%22%2C%22password%22%3A%22%22%7D; acf_auth=a5d318I%2FCSCK1smYvadE0jQHqNOQXEaY3UFO%2BtJS6bDCYOme7Rj%2BsgxnR4AtKHi36NSn54GGddsEf6B%2B2AOTpxTzLHNtWRfCvm%2F%2FAgN5WCEG%2B2ryylwLyf%2B2; dy_auth=8f9fckAvmtVroGJXGEM5IJFQkbJ%2FNtxV7HdwsogTf0JuWrQTAfefyw6rAP%2BZA2vgQLDNiMrZe6RcMiMRqfcLHQl3nFRg8%2BbEBChPAzBipbFBXqOkQOkRnstl; wan_auth37wan=80dadbbdf103%2FLQwWdzs8x38Vc3k9EOqwoOZK%2F2DEabKzEzp1xs3ZOtxew935GIE4jgxn8sfgsUkdpWwdkifesWDxHU%2FZSxl6myOkISOq073bvzlKA; acf_uid=16797974; acf_username=qq_RhS1ohsf; acf_nickname=%E6%AD%A3%E7%BB%8F%E7%9A%84kirito; acf_own_room=1; acf_groupid=1; acf_phonestatus=1; acf_ct=0; acf_ltkid=74690331; acf_biz=1; acf_stk=298e5ce22b3ced1a; acf_avatar=//apic.douyucdn.cn/upload/avatar_v3/202307/6c352ea1170e4658a60eb38361a0d7c7_; PHPSESSID=t05emvtd2rf2vcomck0fuluml2; cvl_csrf_token=21964e67dad24efd8464286138f399c4".into());
                seam_core::live::douyu::Client {}
                    .get(room_id, Some(header))
                    .await?
            }
            "douyin" => {
                let mut header: HashMap<String, String> = HashMap::new();
                header.insert("Cookie".into(), "ttwid=1%7C-JnlRBb3N9mmuBto1NUmxIjQ8UKafKVYv4PmQvXHlew%7C1678532949%7C4bb5459d8bfb044fd2ba3ee2f746d3d534f6af642ed3e4d033d2fde86111b7db; __live_version__=%221.1.1.4094%22; xgplayer_user_id=663738601074; odin_tt=f81b9e75632db03ad4e5dd5f77ec5977cdfcae2893d1540a257007a580cfaf9bda3513d655e787bfa5181648695243230263777060f0a8167f9b6de7c856a471fc0cba6a89d453be6b479ea29ddea4cc; pwa2=%223%7C0%22; d_ticket=ebb9cb92c67873aa5e3a2ccb791868a7cd472; n_mh=gA8SM0JgC29QGaLMCLb4Cqfpt4oleUbGa-Uf6JNY5nc; sid_guard=9e4ee7553db7487…ignature=_02B4Z6wo00f01f2ljxgAAIDA4eYxpj2RhuX9lIuAABpszhxF11gc-CFiN7VF.qaIM.rfTfxT2e3lfnX0624OL0RN4bkVDecaHe0iO8.9uED-xgwWxc9.3Mx9fizAmaIUXgnlZbOVUFGH.9J924; webcast_leading_last_show_time=1694699628930; webcast_leading_total_show_times=1; download_guide=%223%2F20230914%2F0%22; msToken=cmP6C_hYnJLs2WYoMxOiHEoI10Wi0GU41twi5iPtQd-L9_1ysL58YZBYvKZ8dwZfYxxWphIcwHVIgR7unWNIRYyHe6KyOz59gFzzy_mF-561bW20rHaQ0g==; tt_scid=0Cwri0fde80f2UsUrgLgJneTKFlfVQKm7t9yIyMzececp5W77RoGUMkLOJ0RC5i86f1e; webcast_local_quality=sd".into());
                seam_core::live::douyin::Client {}
                    .get(room_id, Some(header))
                    .await?
            }
            "huya" => seam_core::live::huya::Client {}.get(room_id, None).await?,
            _ => {
                return Err(anyhow!("unsupport {}", platform));
            }
        };

        Ok(Self {
            rid: node.rid.clone(),
            title: node.title.clone(),
            cover: node.cover.clone(),
            anchor: node.anchor.clone(),
            head: node.head.clone(),
            platform: platform.into(),
            show_type: if node.urls.is_empty() {
                Some(ShowType::Off)
            } else {
                Some(ShowType::On(node))
            },
            cover_path: None,
            head_path: None,
        })
    }

    pub fn to_card_info(&self) -> anyhow::Result<crate::LiveCardInfo> {
        Ok(crate::LiveCardInfo {
            anchor: self.anchor.clone().into(),
            avatar: cache::load_img(self.head_path.clone().ok_or(anyhow!("some"))?)
                .map_err(|e| anyhow!("load img {:?}", e))?,
            cover: cache::load_img(self.cover_path.clone().ok_or(anyhow!("some"))?)
                .map_err(|e| anyhow!("load img {:?}", e))?,
            title: self.title.clone().into(),
            urls: match self.show_type.clone() {
                Some(st) => st.get_urls_as_model(),
                None => VecModel::from_slice(&[]),
            },
        })
    }
}

#[derive(Debug, Clone)]
pub enum ShowType {
    // 开播
    On(Node),
    // 未开播
    Off,
}

impl ShowType {
    pub fn get_urls_as_model(&self) -> ModelRc<SharedString> {
        let v = VecModel::default();
        match self {
            ShowType::On(node) => node.urls.iter().for_each(|url| {
                v.push(url.url.clone().into());
            }),
            ShowType::Off => {}
        }
        ModelRc::from(Rc::new(v))
    }
}
