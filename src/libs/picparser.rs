//! 解析图片，获得答案

use anyhow::{anyhow, Result};
use bytes::Bytes;
use dssim::{Dssim, ToRGBAPLU, RGBAPLU};
use imgref::{Img, ImgVec};
use reqwest::Client;
use serde::Deserialize;
use std::fmt::Display;

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
            self.img_bytes = Some(b);
        }
        Ok(())
    }

    /// 与答案相比较
    pub async fn compare_with_answers(
        &mut self,
        answers: &mut [Answer],
        client: &Client,
        limit: f64,
    ) -> Result<Answer> {
        let attr = Dssim::new();
        self.get_img(client).await?;
        let mut an = None;
        let mut min_c = f64::MAX - 0.1;
        log::debug!("设置的阈值: {}", limit);
        let Some(ref ori) = self.img_bytes else {
            return Err(anyhow!("无法获取题图"));
        };
        let Some(orig) = attr.create_image(&load_img(ori)?) else {
            return Err(anyhow!("无法获取题图的ssimimg"));
        };
        for i in answers.iter_mut() {
            if i.get_img(client).await.is_err() {
                log::debug!("无法获取海报: {}", i.name);
                continue;
            } else {
                let Some(ref pic) = i.img_bytes else {
                    continue;
                };
                let Ok(other) = &load_img(pic) else {
                    log::debug!("无法获取选项图的img");
                    continue;
                };
                let Some(modif) = attr.create_image(other) else {
                    continue;
                };
                let (co, _) = attr.compare(&orig, modif);

                log::debug!("比较结果: {:.2}", co);
                if co <= limit && co < min_c {
                    an = Some(i.clone());
                    min_c = co.into();
                    continue;
                }
            }
        }

        match an {
            None => Err(anyhow!("所有比较均失败了")),
            Some(a) => {
                log::info!("最高相似: {:.2}", min_c);
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
        log::debug!("获取到的豆瓣信息: {}", data);
        self.img_url = Some(data.img);
        let Some(ref url) = self.img_url else {
            return Err(anyhow!("无法获取图片"));
        };
        let b = client.get(url).send().await?.bytes().await?;
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
    img: String,
    title: String,
    sub_title: Option<String>,
}

impl Display for DouBanData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DoubanData[{}-{}-{}]",
            self.title,
            self.sub_title.is_some(),
            self.img
        )
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

    log::debug!("豆瓣数据: {}个", res.len());

    if let Some(d) = res.into_iter().next() {
        Ok(d)
    } else {
        Err(anyhow!("无有效数据"))
    }
}

fn load_img(m_b: &Bytes) -> Result<ImgVec<RGBAPLU>> {
    let img = lodepng::decode32(m_b)?;
    Ok(Img::new(img.buffer.to_rgbaplu(), img.width, img.height))
}
