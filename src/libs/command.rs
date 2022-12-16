//! 一些常量定义
use clap::crate_name;
use lazy_static::lazy_static;
use platform_dirs::AppDirs;
use std::path::{Path, PathBuf};

/// 配置文件名
pub const CONFIG_FILENAME: &str = "config.toml";

/// 链接们
pub mod tjurls {
    /// 登陆链接
    pub const LOGIN: &str = "https://tjupt.org/login.php";
    /// 签到页面
    pub const ATTENDANCE: &str = "https://tjupt.org/attendance.php";
    /// post的登陆url
    pub const TAKELOGIN: &str = "https://tjupt.org/takelogin.php";
}

/// 文件位置配置
pub struct Dirs {
    config_dir: PathBuf,
    config_path: PathBuf,
    status_dir: PathBuf,
}

impl Dirs {
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn state_dir(&self) -> &Path {
        &self.status_dir
    }
}

lazy_static! {
    pub static ref DIRS: Dirs = {
        let appdir = AppDirs::new(Some(crate_name!()), false).unwrap();

        Dirs {
            config_path: appdir.config_dir.join(CONFIG_FILENAME),
            config_dir: appdir.config_dir,
            status_dir: appdir.state_dir,
        }
    };
}
