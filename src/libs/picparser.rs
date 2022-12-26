//! 解析图片，获得答案

use anyhow::{anyhow, Result};
use bytes::Bytes;
use dssim::{Dssim, DssimImage, ToRGBAPLU};
use image::ImageFormat;
use imgref::Img;
use load_image::ImageData;
use reqwest::Client;
use serde::Deserialize;
use std::{fmt::Display, io::Cursor};

/// 验证码
pub struct Kaptcha {
    pub url: String,
    pub img_bytes: Option<Bytes>,
}

impl Kaptcha {
    pub fn new(url: String) -> Self {
        Self {
            url,
            img_bytes: None,
        }
    }

    /// 获取图片，并且保存在内存里
    async fn get_img(&mut self, client: &Client) -> Result<()> {
        if self.img_bytes.is_none() {
            let b = client.get(self.url.as_str()).send().await?.bytes().await?;
            let b = reseize_pic(b)?;
            self.img_bytes = Some(b);
        }
        Ok(())
    }

    /// 与答案相比较
    pub async fn compare_with_answers(
        &mut self,
        answers: &mut [Answer],
        client: &Client,
        limit_min: f64,
    ) -> Result<Answer> {
        let mut attr = Dssim::new();
        attr.set_scales(&[100.0, 100.0]);
        self.get_img(client).await?;
        let mut an = None;
        let mut min_score = f64::MAX - 0.1;
        // log::debug!("设置的阈值: {}", limit);
        let Some(ref ori) = self.img_bytes else {
            return Err(anyhow!("无法获取题图"));
        };
        let Ok(orig) = load_img(&attr, ori) else {
            return Err(anyhow!("无法获取题图的ssimimg"));
        };
        for i in answers.iter_mut() {
            if let Err(e) = i.get_img(client).await {
                log::warn!("无法获取海报: {}, Err: {}", i.name, e);
                continue;
            } else {
                let Some(ref pic) = i.img_bytes else {
                    log::debug!("无法获取选项图的img");
                    continue;
                };
                let Ok(modif) = &load_img(&attr,pic) else {
                    log::debug!("无法获取选项图的img");
                    continue;
                };
                let (score, _) = attr.compare(&orig, modif);

                // log::debug!("比较结果: {:.2}", co);
                if score <= limit_min && score < min_score {
                    an = Some(i.clone());
                    min_score = score.into();
                    continue;
                }
            }
        }

        match an {
            None => Err(anyhow!("所有比较均失败了")),
            Some(a) => {
                log::info!("最高相似: {:.2}", min_score);
                Ok(a)
            }
        }
    }
}

impl From<String> for Kaptcha {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone)]
pub struct Answer {
    pub name: String,
    pub value: String,
    pub img_url: Option<String>,
    pub img_bytes: Option<Bytes>,
}

impl Answer {
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            img_url: None,
            img_bytes: None,
        }
    }

    pub async fn get_img(&mut self, client: &Client) -> Result<()> {
        let data = get_douban_data(&self.name, client).await?;
        // log::debug!("获取到的豆瓣信息: {}", data);
        self.img_url = Some(data.img);
        let Some(ref url) = self.img_url else {
            return Err(anyhow!("无法获取图片"));
        };
        let b = client.get(url).send().await?.bytes().await?;

        let b = reseize_pic(b)?;

        self.img_bytes = Some(b);
        Ok(())
    }
}

impl From<(String, String)> for Answer {
    fn from(value: (String, String)) -> Self {
        Self::new(value.0, value.1)
    }
}

#[derive(Deserialize)]
struct DouBanData {
    /// 图片链接
    img: String,
    title: String,
    sub_title: Option<String>,
}

impl Display for DouBanData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.sub_title {
            None => write!(f, "DoubanData[{}]", self.title),
            Some(subtitle) => write!(f, "DoubanData[{}-{}]", self.title, subtitle),
        }
    }
}

async fn get_douban_data(name: &str, client: &Client) -> Result<DouBanData> {
    let res: Vec<DouBanData> = client
        .get("https://movie.douban.com/j/subject_suggest")
        .query(&[("q", name)])
        .send()
        .await?
        .json()
        .await?;

    // log::debug!("豆瓣数据: {}个", res.len());

    if let Some(d) = res.into_iter().next() {
        Ok(d)
    } else {
        Err(anyhow!("无法获取豆瓣数据"))
    }
}

fn load_img(attr: &Dssim, m_b: &Bytes) -> Result<DssimImage<f32>> {
    let img = load_image::load_data(m_b)?;

    let res = match img.bitmap {
        ImageData::RGB8(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height))
        }
        ImageData::RGB16(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height))
        }
        ImageData::RGBA8(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height))
        }
        ImageData::RGBA16(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height))
        }
        ImageData::GRAY8(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height))
        }
        ImageData::GRAY16(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height))
        }
        ImageData::GRAYA8(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height))
        }
        ImageData::GRAYA16(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height))
        }
    };
    match res {
        None => Err(anyhow!("error!!!")),
        Some(i) => Ok(i),
    }
}

/// 设置图片尺寸
fn reseize_pic(pic1: Bytes) -> Result<Bytes> {
    let mut reader = image::io::Reader::new(Cursor::new(pic1));
    reader.set_format(image::ImageFormat::Jpeg);
    let img = reader.decode()?;

    let img = image::imageops::resize(&img, 120, 200, image::imageops::FilterType::Nearest);

    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)?;

    Ok(buf.into())
}
