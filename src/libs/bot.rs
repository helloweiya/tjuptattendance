//! 主要逻辑

use crate::{
    command::DIRS,
    config::{ConfigFile, UserConfig},
};
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use reqwest::{header::HeaderMap, Client, ClientBuilder, Url};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use std::{
    fs::{remove_dir_all, File},
    io::{BufReader, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use time::Time;

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

    // async fn

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

    /// 登陆
    async fn login(&self) -> Result<()> {
        // 尝试加载cookie
        let _ = self.load_cookie();
        // TODO
        let res = self
            .client
            .get("https://www.baidu.com/")
            .send()
            .await?
            .text()
            .await?;
        log::debug!("{} 获得的数量: {}", self.config.id(), res.len());

        Ok(())
    }

    /// 签到一次
    async fn att_onece_now(&self) -> Result<()> {
        // TODO
        self.login().await?;
        Ok(())
    }

    /// 签到
    ///
    /// 不会检查是否开启
    pub async fn att_now(&self) -> Result<()> {
        let retry_times = self.config.retry();
        for i in 0..retry_times {
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
        Err(anyhow!("签到失败"))
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

    pub async fn ask_url<S>(&self, url: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let url: Url = url.as_ref().parse()?;

        let result = self.client.get(url).send().await?.text().await?;

        Ok(result)
    }
}

impl TjuPtUser {
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

            // log::debug!("保存cookie: {}", self.config.id(),);
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
                u.delay_when_read(g_conf);
                if u.enable() {
                    Some(TjuPtUser::from_config(u, Some(DIRS.state_dir())))
                } else {
                    None
                }
            })
            .collect::<Vec<TjuPtUser>>();

        // 签到
        // TODO 这里应该不是马上签到
        att_all_now(users).await;
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
            log::error!("签到失败: {}", e);
            continue;
        }
    }
}

/// 批量签到-定时
async fn att_all_on_time(users: Vec<TjuPtUser>) {
    att_all_now(users).await;
    log::debug!("按照配置文件定时签到");
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
