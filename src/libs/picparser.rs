//! 解析图片，获得答案

use anyhow::{anyhow, Result};
use bytes::Bytes;
use reqwest::Client;

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
        answers: Vec<Answer>,
        client: &Client,
        limit: f32,
    ) -> Result<Answer> {
        self.get_img(client).await?;
        let an = None;
        log::debug!("设置的阈值: {}%", limit);
        for mut i in answers.into_iter() {
            if i.get_img(client).await.is_err() {
                log::debug!("无法获取海报: {}", i.name);
                continue;
            } else {
                todo!()
            }
        }

        match an {
            None => Err(anyhow!("所有比较均失败了")),
            Some(a) => Ok(a),
        }
    }
}

impl From<String> for Kaptcha {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

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

    async fn get_img_url(&mut self) -> Result<()> {
        self.img_url = Some("asd".into());
        Ok(())
    }

    pub async fn get_img(&mut self, client: &Client) -> Result<()> {
        let _r = self.get_img_url().await;
        let Some(ref url) = self.img_url else {
            return Err(anyhow!("无法获取图片"));
        };
        let b = client.get(url).send().await?.bytes().await?;
        self.img_bytes = Some(b);
        Ok(())
    }
}
