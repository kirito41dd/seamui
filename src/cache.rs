use std::{
    any,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use image::{self, Rgba};
use load_image::export::rgb::{ComponentMap, RGBA8};
use log::info;
use slint::{Image, LoadImageError};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::config::cache_path;

// 返回本地文件路径
pub async fn download_img_to_cache(url: &str) -> anyhow::Result<PathBuf> {
    let url = reqwest::Url::parse(url)?;
    let mut name = match url
        .path_segments()
        .ok_or(anyhow!("path_segments"))?
        .into_iter()
        .rev()
        .find(|v| {
            v.to_lowercase().ends_with(".jpg")
                || v.to_lowercase().ends_with(".png")
                || v.to_lowercase().ends_with(".avif")
        }) {
        Some(v) => v.to_string(),
        None => {
            format!("{:x}.png", md5::compute(url.path().to_string()))
        }
    };

    let mut need_convert = true;
    if name.ends_with(".avif") {
        name.push_str(".png");
        need_convert = true;
    }
    let mut path = cache_path();
    path.push(name);

    let metadata = tokio::fs::metadata(path.clone()).await;

    if metadata.is_ok() {
        return Ok(path);
    }

    // 下载并保存
    _ = tokio::fs::create_dir_all(cache_path()).await;

    let resp = reqwest::get(url).await?;
    let content = resp.bytes().await?;

    // 必须这么来一遍，斗鱼返回的图片和后缀格式对不上， 统一rgba转换下
    let path_clone = path.clone();
    tokio::task::spawn_blocking(move || {
        let img = load_image::load_data(&content)?;
        let img = image_load_img_to_png(img)?;
        img.save(path_clone.clone())?;
        return anyhow::Ok(());
    })
    .await??;

    Ok(path)
}

pub fn load_img<P: AsRef<Path>>(path: P) -> Result<Image, LoadImageError> {
    Image::load_from_path(path.as_ref())
}

pub fn image_load_img_to_png(
    img: load_image::Image,
) -> anyhow::Result<image::ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let w = img.width;
    let h = img.height;
    let img = img.into_imgvec();

    let mut img = match img {
        load_image::export::imgref::ImgVecKind::RGB8(img) => {
            img.map_buf(|buf| buf.into_iter().map(|px| px.alpha(255)).collect())
        }
        load_image::export::imgref::ImgVecKind::RGBA8(img) => img,
        load_image::export::imgref::ImgVecKind::RGB16(img) => img.map_buf(|buf| {
            buf.into_iter()
                .map(|px| px.map(|c| (c >> 8) as u8).alpha(255))
                .collect()
        }),
        load_image::export::imgref::ImgVecKind::RGBA16(img) => img.map_buf(|buf| {
            buf.into_iter()
                .map(|px| px.map(|c| (c >> 8) as u8))
                .collect()
        }),
        load_image::export::imgref::ImgVecKind::GRAY8(img) => img.map_buf(|buf| {
            buf.into_iter()
                .map(|g| {
                    let c = g.0;
                    RGBA8::new(c, c, c, 255)
                })
                .collect()
        }),
        load_image::export::imgref::ImgVecKind::GRAY16(img) => img.map_buf(|buf| {
            buf.into_iter()
                .map(|g| {
                    let c = (g.0 >> 8) as u8;
                    RGBA8::new(c, c, c, 255)
                })
                .collect()
        }),
        load_image::export::imgref::ImgVecKind::GRAYA8(img) => img.map_buf(|buf| {
            buf.into_iter()
                .map(|g| {
                    let c = g.0;
                    RGBA8::new(c, c, c, g.1)
                })
                .collect()
        }),
        load_image::export::imgref::ImgVecKind::GRAYA16(img) => img.map_buf(|buf| {
            buf.into_iter()
                .map(|g| {
                    let c = (g.0 >> 8) as u8;
                    RGBA8::new(c, c, c, (g.1 >> 8) as u8)
                })
                .collect()
        }),
    };

    let mut ps = Vec::with_capacity(h * w * 4);
    img.into_iter().for_each(|v| {
        ps.push(v.r);
        ps.push(v.g);
        ps.push(v.b);
        ps.push(v.a);
    });

    let ret = image::ImageBuffer::<image::Rgba<u8>, Vec<_>>::from_vec(w as u32, h as u32, ps)
        .ok_or(anyhow!("from  vec"))?;

    Ok(ret)
}

#[cfg(test)]
mod test {
    use image::Pixel;
    use load_image::export::rgb::{ComponentMap, RGBA8};

    use crate::cache::{download_img_to_cache, image_load_img_to_png};

    #[tokio::test]
    async fn test_download() {
        let r = download_img_to_cache(
            "http://i0.hdslb.com/bfs/live/85747a06c4cf64559ab07b0014e6607ab64bcfb4.jpg",
        )
        .await;
        println!("{:?}", r);
        let r = download_img_to_cache(
            "http://i0.hdslb.com/bfs/live/85747a06c4cf64559ab07b0014e6607ab64bcfb4.jpg",
        )
        .await;
        println!("{:?}", r);
    }

    #[test]
    fn image_ops() {
        // let data = std::fs::read::<&str>(
        //     r#"C:\Users\kirito\AppData\Local\seamui2\cache\286138_src_2329.avif"#.into(),
        // )
        // .unwrap();
        let i = load_image::load_path(
            r#"C:\Users\kirito\AppData\Local\seamui2\cache\286138_src_2329.avif"#,
        )
        .unwrap();

        println!("{} {} {:?}", i.width, i.height, i.meta);
        let ret = image_load_img_to_png(i).unwrap();
        ret.save("test.png");

        // image::load_from_memory(buffer)
        // let img =
        //     image::open(r#"C:\Users\kirito\AppData\Local\seamui2\cache\286138_src_2329.avif"#)
        //         .unwrap();
    }
}
