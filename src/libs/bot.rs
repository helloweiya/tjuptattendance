//! 主要逻辑

use crate::config::EmailConfig;
use crate::{
    command::{tjurls, DIRS},
    config::{ConfigFile, UserConfig},
};
use crate::{email_bot, picparser};
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use reqwest::{header::HeaderMap, redirect, Client, ClientBuilder};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use scraper::{Html, Selector};
use std::{
    fs::{remove_dir_all, File},
    io::{BufReader, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

lazy_static! {
    static ref HEADER: HeaderMap = {
        let mut head = HeaderMap::new();
        head.insert(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
            AppleWebKit/537.36 (KHTML, like Gecko) \
            Chrome/100.0.0.0 Safari/537.36"
                .parse()
                .unwrap(),
        );
        head
    };

    // //input[@type="radio"]
    // //input[@type="submit"]
    static ref INPUT_RADIO_SELE: Selector = Selector::parse(r#"input[type="radio"]"#).unwrap();
    // static ref INPUT_SUBMIT_SELE: Selector = Selector::parse(r#"input[type="submit"]"#).unwrap();

    static ref TD: Selector = Selector::parse(r#"td[id="outer"]"#).unwrap();
    static ref IMG: Selector = Selector::parse("img").unwrap();

}

/// tjupt user
#[derive(Debug)]
pub struct TjuPtUser {
    config: UserConfig,
    client: Client,
    cookie: Arc<CookieStoreMutex>,
    cookie_path: Option<PathBuf>,
}

impl TjuPtUser {
    pub fn from_config<P>(userconfig: UserConfig, status_dir: Option<P>) -> Self
    where
        P: AsRef<Path>,
    {
        let cookie_path =
            status_dir.map(|p| p.as_ref().join(format!("{}_cookie.json", userconfig.id())));

        Self::new(userconfig, cookie_path)
    }

    fn new<P>(config: UserConfig, cookie_path: Option<P>) -> Self
    where
        P: AsRef<Path>,
    {
        let cookie = Arc::new(CookieStoreMutex::default());

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(5))
            .cookie_store(true)
            .cookie_provider(cookie.clone())
            .connection_verbose(false)
            .default_headers(HEADER.clone())
            .redirect(redirect::Policy::limited(3))
            .build()
            .unwrap();

        let cookie_path = match cookie_path {
            None => None,
            Some(p) => {
                let p = p.as_ref();
                Some(p.into())
            }
        };

        Self {
            client,
            config,
            cookie,
            cookie_path,
        }
    }

    /// 加载cookie
    ///
    /// 如果未设置cookiepath也返回Ok
    fn load_cookie(&self) -> Result<()> {
        if let Some(ref cookie_path) = self.cookie_path {
            let cookie_path = cookie_path.as_path();
            if cookie_path.is_file() {
                let file = File::open(cookie_path).map(BufReader::new)?;
                let cookie = CookieStore::load_json(file).map_err(|e| anyhow!("{}", e))?;
                let mut lock = self.cookie.lock().map_err(|e| anyhow!("{}", e))?;
                *lock = cookie;

                // log::debug!("加载cookie成功 {}", cookie_path.display());
            } else {
                // log::debug!("本地cookie不存在 {}", cookie_path.display());
                return Err(anyhow!("coookie 不存在"));
            }
        }
        Ok(())
    }

    /// 在不加载cookie的情况下登陆
    ///
    /// 返回签到页面的String
    async fn login(&self) -> Result<String> {
        let _r = self.client.get(tjurls::LOGIN).send().await?;
        if !self
            .client
            .post(tjurls::TAKELOGIN)
            // .query(&[("returnto", "attendance.php")])
            .form(&[
                ("username", self.config.id()),
                ("password", self.config.pwd()),
                ("logout", "7days"),
                ("returnto", "attendance.php"),
            ])
            .send()
            .await?
            .status()
            .is_success()
        {
            return Err(anyhow!("请检查网络"));
        }
        let req = self.client.get(tjurls::ATTENDANCE).send().await?;
        if !req.url().as_str().contains("login.php") {
            let content = req.text().await?;
            Ok(content)
        } else {
            Err(anyhow!("发送登陆请求失败"))
        }
    }

    /// 登陆
    /// 在这之前加载过cookie了
    async fn get_att_html(&self) -> Result<String> {
        let req = self.client.get(tjurls::ATTENDANCE).send().await?;
        // 先获取签到页面，检查链接
        if req.url().as_str().contains("login.php") {
            // 如果重定向了说明需要登陆
            self.login().await
        } else {
            // 如果成功，那么就直接ok
            let content = req.text().await?;
            Ok(content)
        }
    }

    /// 签到一次
    ///
    /// 但是不在这里加载cookie
    /// 也不在这登录
    async fn att_onece_now(&self) -> Result<()> {
        let Ok(html) = self.get_att_html().await else {
            return Err(anyhow!("{} 登录失败", self.config.id()));
        };

        // 解析网页，获取选项信息
        let doc = Html::parse_document(&html);

        // //input[@type="radio"]s
        let radio = doc.select(&INPUT_RADIO_SELE);

        // 选项们
        let answers = radio
            .into_iter()
            .filter_map(|e| {
                let Some(name) = e.next_sibling()  else {
                return None;
            };
                let Some(name) = name.value().as_text() else {
                return None;
            };
                // let name = name.to_string();

                let Some(value) = e.value().attr("value") else {
                return None;
            };

                Some((name.to_string(), value.to_string()))
            })
            .collect::<Vec<_>>();

        // 图片
        let Some(img) = doc.select(&TD).next().
            and_then(|e|e.select(&IMG).next())
            .and_then(|e| e.value().attr("src") ) 
        else {
            return Err(anyhow!("无法定位图片"));
        };

        let img_url = format!("https://tjupt.org{}", img);

        // log::debug!("获取的图片链接: {}", img_url);
        // 这里检查一下图片应该是jpg结尾的
        if !img_url.ends_with(".jpg") {
            return Err(anyhow!("无法获取jpg格式图片"));
        }

        // for (x, y) in answers.iter() {
        //     log::debug!("选项: {}, {}", x, y);
        // }

        if answers.is_empty() {
            // 如果是空的，说明签到完了，或者需要补签
            return Err(anyhow!("无法找到选项，可能已经签到，或需要补签"));
        }

        // 获取结果
        let mut answers: Vec<_> = answers.into_iter().map(picparser::Answer::from).collect();
        let mut kaptcha = picparser::Kaptcha::new(img_url);

        let result = tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                kaptcha
                    .compare_with_answers(&mut answers, &self.client, 0.1)
                    .await
            })
        })?;

        log::info!("结果是: {}", result.name);

        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current()
                .block_on(async move { self.post_answer(&result.value).await })
        })
    }

    /// 签到
    ///
    /// 不会检查是否开启
    ///
    /// 尝试加载cookie一次
    ///
    /// 并立即签到
    pub async fn att_now(&self, enable_email: bool, email_config: Arc<EmailConfig>) -> Result<()> {
        // 这里加载一次cookie就好
        let _res = self.load_cookie();

        let retry_times = self.config.retry();
        for i in 0..retry_times {
            // 此时登陆
            // if let Err(e) = self.get_att_html().await {
            //     log::debug!("登录失败: {}, Error: {}", self.config.id(), e);
            //     continue;
            // }
            if let Err(e) = self.att_onece_now().await {
                log::debug!(
                    "{} 签到失败 {}/{} Error: {}",
                    self.config.id(),
                    i + 1,
                    retry_times,
                    e
                );
                continue;
            } else {
                log::info!("签到成功: {}", self.config.id());
                return Ok(());
            }
        }

        if enable_email {
            if let Some(rec) = self.config.email() {
                if let Err(e) = email_bot::send_email(
                    &email_config,
                    rec,
                    format!("{} 签到失败", self.config.id()).as_str(),
                ) {
                    log::error!("邮件发送失败!, Err: {}", e);
                }
            }
        }

        Err(anyhow!("签到失败: {}", self.config.id()))
    }

    /// 清除cookie
    pub fn clear_cookie(&self) -> Result<()> {
        let Ok(mut lock) = self.cookie.lock() else {
            return Err(anyhow!("无法获取锁"));
        };
        lock.clear();
        Ok(())
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    async fn post_answer(&self, value: &str) -> Result<()> {
        let data = &[("answer", value), ("submit", "提交")];
        let r = self
            .client
            .post(tjurls::ATTENDANCE)
            .form(data)
            .send()
            .await?
            .text()
            .await?;
        if r.contains("签到成功") {
            Ok(())
        } else {
            Err(anyhow!("签到失败"))
        }
    }

    /// 保存 cookie 到 cookie_path
    ///
    /// 不为 cookiepath 为None 的保存
    pub fn save_cookie(&self) -> Result<()> {
        if let Some(ref cookie_path) = self.cookie_path {
            let Ok(mut file) = File::create(cookie_path) else {
                return Err(anyhow!("无法创建cookie文件: {}，请尝试 `--init`", cookie_path.display()));
            };

            {
                let lock = self.cookie.lock().map_err(|e| anyhow!("无法获取锁{}", e))?;

                lock.save_json(&mut file)
                    .map_err(|e| anyhow!("无法写入cookie error: {}", e))?;
            }
        }
        Ok(())
    }
}

impl Drop for TjuPtUser {
    fn drop(&mut self) {
        if let Err(e) = self.save_cookie() {
            log::warn!("{}", e);
        }
    }
}

/// 初始化
///
/// 创建文件夹，及默认配置文件
pub fn initialization() -> Result<()> {
    // 创建文件夹
    for p in [DIRS.config_dir(), DIRS.state_dir()] {
        if !p.is_dir() {
            log::info!("创建文件夹: {}", p.display());
            std::fs::create_dir_all(p)?;
        }
    }

    // 创建默认配置文件
    let config_path = DIRS.config_path();
    if !config_path.is_file() {
        log::info!("创建默认配置文件: {}", config_path.display());
        let mut file = File::create(config_path)?;
        let _ = file.write(toml::to_string(&ConfigFile::default())?.as_bytes())?;
    }

    Ok(())
}

/// 卸载
pub fn uninstall() -> Result<()> {
    for p in [DIRS.config_dir(), DIRS.state_dir()] {
        if p.is_dir() {
            log::info!("删除文件夹及所包含文件: {}", p.display());
            remove_dir_all(p)?;
        }
    }

    Ok(())
}

/// 解析命令行参数，并且运行
pub async fn attendance() -> Result<()> {
    let mat = crate::cliparser::cli_parser()?;

    let config_path: &String = mat.get_one("file").unwrap();

    let enable_email = mat.get_flag("email");

    if mat.get_flag("init") {
        // 如果是初始化
        crate::bot::initialization()?;
    } else if mat.get_flag("uninstall") {
        // 如果是卸载
        uninstall()?;
    } else if mat.contains_id("user") {
        // 如果制定了user，那么就使用这个user
        let users: Vec<&String> = mat.get_many("user").unwrap().collect();
        let users_num = users.len() / 2;
        let retry: u8 = *mat.get_one("retry").unwrap();
        let mut users_vec = vec![];
        for i in 0..users_num {
            let Some(user_id) = users.get(2*i)
                else {continue;};
            let Some(user_pwd) = users.get(2*i+1)
                else {continue;};
            let user = UserConfig::new(
                true,
                user_id.to_string(),
                user_pwd.to_string(),
                None,
                Some(retry),
            );

            users_vec.push(TjuPtUser::from_config::<&Path>(user, None));
        }

        // 开始马上签到
        let email_config = Arc::new(EmailConfig::default());
        att_all_now(users_vec, false, email_config).await;
    } else if let Some(config_mat) = mat.subcommand_matches("config") {
        // 如果是配置文件
        let config_path: &String = config_mat.get_one("file").unwrap();
        let config_path = Path::new(config_path);

        let mut config_file = {
            if config_path.is_file() {
                ConfigFile::new_from(config_path)?
            } else {
                ConfigFile::default()
            }
        };

        if config_mat.contains_id("adduser") {
            // 如果是增加用户
            let users: Vec<&_> = config_mat
                .get_many::<String>("adduser")
                .unwrap()
                .map(|e| e.as_str())
                .collect();

            let users = get_users_vec(users);
            let users = users
                .into_iter()
                .map(|(id, pwd)| UserConfig::new(true, id.into(), pwd.into(), None, None))
                .collect::<Vec<UserConfig>>();
            config_file.addusers(users);
            config_file.write_to_file(config_path)?;
        }
        if config_mat.contains_id("rmuser") {
            // 如果是删除用户
            config_file.rmusers(
                config_mat
                    .get_many("rmuser")
                    .unwrap()
                    .map(|s: &String| s.as_str())
                    .collect(),
            );
            config_file.write_to_file(config_path)?;
        }
        if config_mat.get_flag("show") {
            // 打印配置信息
            println!("配置文件位置: {}", config_path.display());
            println!("配置文件信息: {}", config_file);
        }
    } else {
        // 其他情况，使用配置文件直接运行
        let config_file = ConfigFile::new_from(config_path)?;
        let g_conf = config_file.gloablconfig();
        let users = config_file
            .get_users()
            .into_iter()
            .filter_map(|mut u| {
                u.update_retry(g_conf);
                if u.enable() {
                    Some(TjuPtUser::from_config(u, Some(DIRS.state_dir())))
                } else {
                    None
                }
            })
            .collect::<Vec<TjuPtUser>>();

        // 签到
        let email_config = Arc::new(config_file.get_email_config());
        att_all_now(users, enable_email, email_config).await;
    }
    Ok(())
}

/// 批量签到
async fn att_all_now(users: Vec<TjuPtUser>, enable_email: bool, email_config: Arc<EmailConfig>) {
    // 签到
    let mut hands = vec![];
    for i in users.into_iter() {
        let email_config = email_config.clone();
        hands.push(tokio::spawn(async move {
            i.att_now(enable_email, email_config).await
        }));
    }

    for i in hands.into_iter() {
        let Ok(i) = i.await else {
            continue;
        };

        if let Err(e) = i {
            log::error!("{}", e);
            continue;
        }
    }
}

/// 从user——vec转users
fn get_users_vec(users: Vec<&str>) -> Vec<(&str, &str)> {
    let users_num = users.len() / 2;
    let mut users_res = vec![];
    for i in 0..users_num {
        // let id = users[2*i];
        let Some(id) = users.get(2*i) else {
            continue;
        };
        let Some(pwd) = users.get(2*i+1) else {
            continue;
        };

        users_res.push((*id, *pwd));
    }
    users_res
}
