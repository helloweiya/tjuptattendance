//! 主要逻辑

use crate::{
    command::{tjurls, DIRS},
    config::{ConfigFile, UserConfig},
};
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
use time::{OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

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
    static ref TIMEOFFSET: UtcOffset = UtcOffset::from_hms(8, 0, 0).unwrap();

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

                Some((name, value))
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

        log::debug!("{}", img_url);

        for (x, y) in answers {
            log::debug!("{}, {}", x.to_string().as_str(), y);
        }

        Ok(())
    }

    /// 签到
    ///
    /// 不会检查是否开启
    ///
    /// 尝试加载cookie一次
    ///
    /// 并立即签到
    pub async fn att_now(&self) -> Result<()> {
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
        Err(anyhow!("签到失败: {}", self.config.id()))
    }

    /// 等待到时间点进行签到
    pub async fn att_on_time(&mut self) -> Result<()> {
        // 先进性排序
        self.sort_points_in_time();

        // 这里加载一次cookie就好
        let _res = self.load_cookie();

        let retry_times = self.config.retry();

        // let mut need_wait = true;

        // 等待到时间 (少等2秒)
        tokio::time::sleep(std::time::Duration::
            // from_micros(self.get_next_att_duration().abs().whole_microseconds() as u64 - (self.config.delay() * 1000) as u64)
            from_secs_f64(
            (self.get_next_att_duration().whole_seconds().abs()
                - (self.config.delay() / 1000) as i64) as f64,
        ))
        .await;

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

    /// 对points_in_time进行排序
    pub fn sort_points_in_time(&mut self) {
        if let Some(ref mut points_in_time) = self.config.points_in_time {
            points_in_time.sort();
        }
    }

    /// 获取距离下次签到的时间间隔
    ///
    /// 调用前要进行排序的
    pub fn get_next_att_duration(&self) -> time::Duration {
        let no_dur = time::Duration::seconds_f32(0.0);
        let now = get_date_time();
        let mut next_time = None;

        if let Some(timepoints) = self.config.points_in_time() {
            for point in timepoints {
                if point >= &now.time() {
                    next_time = Some(point);
                    break;
                }
            }

            if let Some(next_time) = next_time {
                // 如果今天有下次签到的时间点
                log::info!("{} 预定时间点: {}", self.config.id(), next_time);
                *next_time - now.time()
            } else {
                // 如果是之前的时间点则是第二天的第一个
                let Some(next_date) = now.date().next_day() else {
                    log::warn!("无法获取下一天的日期");
                    return  no_dur;
                };

                let Some(first_point) = self.config.points_in_time().and_then(|t| t.get(0) ) else {
                    return no_dur;
                };

                let next_date_time =
                    PrimitiveDateTime::new(next_date, *first_point).assume_offset(*TIMEOFFSET);
                log::warn!("{} 选中明天的时间点: {}", self.config.id(), next_date_time);
                next_date_time - now
            }
        } else {
            no_dur
        }
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

    if mat.get_flag("init") {
        // 如果是初始化
        crate::bot::initialization()?;
    } else if mat.get_flag("uninstall") {
        // 如果是卸载
        uninstall()?;
    } else if mat.contains_id("time") {
        // 如果是定时启动
        let users: Vec<&String> = mat.get_many("user").unwrap().collect();
        let users_num = users.len() / 2;
        let retry: u8 = *mat.get_one("retry").unwrap();
        let delay: u32 = *mat.get_one("delay").unwrap();
        let time_points: Vec<&u8> = mat.get_many("time").unwrap().collect();
        let Ok(points_in_time) = Time::from_hms(
            *time_points[0],
            *time_points[1],
            *time_points[2]) else {
            return Err(anyhow!("非法的时间点参数"));
        };

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
                Some(delay),
                Some(vec![points_in_time]),
            );

            users_vec.push(TjuPtUser::from_config::<&Path>(user, None));
        }

        // 开始异步得签到
        att_all_on_time(users_vec).await;
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
                None,
                None,
            );

            users_vec.push(TjuPtUser::from_config::<&Path>(user, None));
        }

        // 开始马上签到
        att_all_now(users_vec).await;
    }
    // TODO 配置文件操作
    else if let Some(config_mat) = mat.subcommand_matches("config") {
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
                .map(|(id, pwd)| {
                    UserConfig::new(true, id.into(), pwd.into(), None, None, None, None)
                })
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
                u.update_delay(g_conf);
                if u.enable() {
                    Some(TjuPtUser::from_config(u, Some(DIRS.state_dir())))
                } else {
                    None
                }
            })
            .collect::<Vec<TjuPtUser>>();

        // 签到
        att_all_on_time(users).await;
    }
    Ok(())
}

/// 批量签到
async fn att_all_now(users: Vec<TjuPtUser>) {
    // 签到
    let mut hands = vec![];
    for i in users.into_iter() {
        hands.push(tokio::spawn(async move { i.att_now().await }));
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

/// 批量签到-定时
async fn att_all_on_time(users: Vec<TjuPtUser>) {
    // 签到
    let mut hands = vec![];
    for mut i in users.into_iter() {
        hands.push(tokio::spawn(async move { i.att_on_time().await }));
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

/// 获取日期时间
pub fn get_date_time() -> OffsetDateTime {
    OffsetDateTime::now_utc().to_offset(*TIMEOFFSET)
}
