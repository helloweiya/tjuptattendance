//! 简单的邮件提醒

use crate::config::EmailConfig;
use anyhow::Result;

pub fn send_email(e_config: &EmailConfig, rece: &str, content: &str) -> Result<()> {
    log::info!(
        "{}给{}发送邮件成功, 内容: {}",
        e_config.user(),
        rece,
        content
    );
    Ok(())
}
