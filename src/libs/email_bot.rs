//! 简单的邮件提醒

use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::config::EmailConfig;
use anyhow::Result;

pub fn send_email(e_config: &EmailConfig, rece: &str, content: &str) -> Result<()> {
    let f_email = format!("TJUPT_BOT <{}>", e_config.sender());
    let t_email = format!("YOU <{}>", rece);

    let email = Message::builder()
        .from(f_email.parse()?)
        .to(t_email.parse()?)
        .subject("TJUPT BOT STATUS")
        .body(String::from(content))?;

    let creds = Credentials::new(e_config.user().into(), e_config.pwd().into());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(e_config.host())?
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => {
            log::debug!("发送邮件成功: {}", t_email);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
