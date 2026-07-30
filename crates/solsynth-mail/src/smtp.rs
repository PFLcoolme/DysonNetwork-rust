//! SMTP 客户端
模块//!
//! 负责 SMTP 连接管理和邮件发送

use anyhow::Result;
use lettre::{
    Message,
    SmtpTransport,
    Transport,
    MessageBuilder,
    address::Address,
    transport::smtp::authentication::Credentials,
};
use tracing::{info, error, warn};

use crate::models::{MailMessage, MailStatus, SmtpConfig};

/// SMTP 客户端
pub struct SmtpClient {
    config: SmtpConfig,
    pool: Vec<SmtpTransport>,
}

impl SmtpClient {
    /// 创建新的 SMTP 客户端
    pub fn new(config: SmtpConfig) -> Self {
        info!("创建 SMTP 客户端: {}:{}", config.host, config.port);
        Self {
            config,
            pool: Vec::new(),
        }
    }

    /// 创建 SMTP 传输
    fn create_transport(&self) -> Result<SmtpTransport> {
        let mut builder = SmtpTransport::builder(&self.config.host)
            .port(self.config.port);

        if self.config.tls {
            builder = builder.credentials(Credentials::new(
                self.config.username.clone(),
                self.config.password.clone(),
            ));
        }

        Ok(builder.build())
    }

    /// 获取可用连接
    pub fn get_connection(&mut self) -> Result<SmtpTransport> {
        if let Some(conn) = self.pool.pop() {
            Ok(conn)
        } else {
            self.create_transport()
        }
    }

    /// 释放连接到连接池
    pub fn release_connection(&mut self, conn: SmtpTransport) {
        if self.pool.len() < self.config.max_connections as usize {
            self.pool.push(conn);
        }
    }

    /// 发送邮件
    pub async fn send(&mut self, mail: &MailMessage) -> Result<()> {
        info!("发送邮件给: {:?}", mail.to);

        let from = format!("{} <{}>", self.config.from_name, self.config.from_address
        
);        let message = Message::builder()
            .from(from.parse()?)
            .to(mail.to[0].parse()?)
            .subject(&mail.subject);

        let message = if !mail.html_body.is_empty() {
            if let Some(text_body) = &mail.text_body {
                message.text_part(text_body.to_string())
                    .html_part(mail.html_body.clone())
                    .unwrap()
            } else {
                message.html(mail.html_body.clone()).unwrap()
            }
        } else {
            message.text(mail.text_body.clone().unwrap_or_default()).unwrap()
        };

        let
 mut transport = self.create_transport()?;        let result = transport.send(&message).await;

        match result {
            Ok(_) => {
                info!("邮件发送成功");
                Ok(())
            }
            Err(e) => {
                error!("邮件发送失败: {}", e);
                Err(anyhow::anyhow!("邮件发送失败: {}", e))
            }
        }
    }

    /// 构建密码重置邮件
    pub fn build_password_reset_email(email: &str, reset_token: &str) -> (String, String) {
        let subject = "重置您的密码".to_string();
        let html_body = format!(
            r#"
            <html>
                <body>
                    <h1>重置密码</h1>
                    <p>您好,</p>
                    <p>我们收到了重置密码的请求。点击以下链接重置您的密码：</p>
                    <a href="{}/reset-password?token={}" style="background-color: #4CAF50; color: white; padding: 14px 20px; text-decoration: none; border-radius: 4px;">重置密码</a>
                    <p>或者访问 https://solsynth.com/reset-password 并输入以下代码：</p>
                    <p><strong>{}</strong></p>
                    <p>此链接将在 30 分钟后过期。</p>
                    <p>如果您没有请求重置密码，请忽略此邮件。</p>
                    <br>
                    <p>Solsynth 团队</p>
                </body>
            </html>
            "#,
            std::env::var("APP_URL").unwrap_or_else(|_| "https://app.solsynth.com".to_string()),
            reset_token,
            reset_token
        );
        (subject, html_body)
    }

    /// 构建邮件验证邮件
    pub fn build_email_verification(email: &str, verification_token: &str) -> (String, String) {
        let subject = "验证您的邮箱".to_string();
        let html_body = format!(
            r#"
            <html>
                <body>
                    <h1>验证邮箱</h1>
                    <p>您好,</p>
                    <p>感谢您的注册！请点击以下链接验证您的邮箱：</p>
                    <a href="{}/verify-email?token={}" style="background-color: #2196F3; color: white; padding: 14px 20px; text-decoration: none; border-radius: 4px;">验证邮箱</a>
                    <p>或者访问以下链接：</p>
                    <p>https://app.solsynth.com/verify-email?token={}</p>
                    <p>此链接将在 24 小时后过期。</p>
                    <br>
                    <p>Solsynth 团队</p>
                </body>
            </html>
            "#,
            std::env::var("APP_URL").unwrap_or_else(|_| "https://app.solsynth.com".to_string()),
            verification_token,
            verification_token
        );
        (subject, html_body)
    }

    /// 构建通知邮件
    pub fn build_notification_email(subject: &str, message: &str, user_name: &str) -> (String, String) {
        let html_body = format!(
            r#"
            <html>
                <body>
                    <h1>{}</h1>
                    <p>您好 {},</p>
                    <p>{}</p>
                    <br>
                    <p>Solsynth 团队</p>
                </body>
            </html>
            "#,
            subject, user_name, message
        );
        (subject.to_string(), html_body)
    }
}