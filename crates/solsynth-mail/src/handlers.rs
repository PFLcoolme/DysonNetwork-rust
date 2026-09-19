//! 邮件服务 API 处理器

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::models::{ApiResponse, SendMailRequest};
use crate::queue::MailQueue;
use crate::smtp::SmtpClient;
use crate::templates::TemplateManager;
use crate::models::{MailMessage, MailType, MailStatus};

/// API 状态
#[derive(Clone)]
pub struct MailState {
    pub smtp_client: std::sync::Arc<tokio::sync::Mutex<SmtpClient>>,
    pub mail_queue: MailQueue,
    pub template_manager: TemplateManager,
}

/// 注册路由
pub fn routes(state: MailState) -> Router {
    Router::new()
        .route("/api/mail/send", post(send_mail))
        .route("/api/mail/stats", get(get_stats))
        .route("/api/mail/queue/process", post(process_queue))
        .with_state(state)
}

/// 发送邮件
async fn send_mail(
    State(state): State<MailState>,
    Json(req): Json<SendMailRequest>,
) -> impl axum::response::IntoResponse {
    // 生成邮件 ID
    let mail_id = Uuid::new_v4();

    // 确定邮件类型
    let mail_type = if req.template_name.is_some() {
        MailType::Notification
    } else {
        MailType::System
    };

    // 创建邮件消息
    let mail_message = MailMessage {
        id: mail_id,
        from: std::env::var("SMTP_FROM_ADDRESS").unwrap_or_else(|_| "noreply@solsynth.com".to_string()),
        to: req.to,
        cc: req.cc.unwrap_or_default(),
        bcc: req.bcc.unwrap_or_default(),
        subject: req.subject,
        html_body: req.html_body,
        text_body: req.text_body,
        attachments: req.attachments.unwrap_or_default(),
        mail_type,
        template_name: req.template_name,
        template_vars: req.template_vars,
        status: MailStatus::Queued,
        retry_count: 0,
        error_message: None,
        sent_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // 加入队列
    let queued_id = state.mail_queue.enqueue(mail_message);

    ApiResponse::success(serde_json::json!({
        "mail_id": queued_id.to_string(),
        "status": "queued",
    }))
}

/// 获取邮件统计
async fn get_stats(
    State(state): State<MailState>,
) -> impl axum::response::IntoResponse {
    let stats = state.mail_queue.get_stats();

    ApiResponse::success(stats)
}

/// 处理队列
async fn process_queue(
    State(state): State<MailState>,
) -> impl axum::response::IntoResponse {
    // 这里简单地触发队列处理
    // 实际实现中应该由后台任务持续处理

    let mut smtp_client = state.smtp_client.lock().await;
    let mut processed = 0u32;

    while let Some((_mail_id, mail)) = state.mail_queue.dequeue() {
        match smtp_client.send(&mail).await {
            Ok(_) => state.mail_queue.mark_sent(&mail.id),
            Err(e) => state.mail_queue.mark_failed(&mail.id, Some(e.to_string())),
        }
        processed += 1;
    }

    ApiResponse::success(serde_json::json!({
        "processed": processed,
    }))
}

/// 构建密码重置邮件
pub fn build_password_reset_message(
    to_email: &str,
    reset_token: &str,
    user_name: &str,
) -> MailMessage {
    let (subject, html_body) = SmtpClient::build_password_reset_email(to_email, reset_token);

    MailMessage {
        id: Uuid::new_v4(),
        from: std::env::var("SMTP_FROM_ADDRESS").unwrap_or_else(|_| "noreply@solsynth.com".to_string()),
        to: vec![to_email.to_string()],
        cc: Vec::new(),
        bcc: Vec::new(),
        subject,
        html_body,
        text_body: None,
        attachments: Vec::new(),
        mail_type: MailType::PasswordReset,
        template_name: Some("password_reset".to_string()),
        template_vars: Some(serde_json::json!({

                       "user_name": user_name, "reset_token": reset_token,
        })),
        status: MailStatus::Queued,
        retry_count: 0,
        error_message: None,
        sent_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// 构建邮箱验证邮件
pub fn build_email_verification_message(
    to_email: &str,
    verification_token: &str,
    user_name: &str,
) -> MailMessage {
    let (subject, html_body) = SmtpClient::build_email_verification(to_email, verification_token);

    MailMessage {
        id: Uuid::new_v4(),
        from: std::env::var("SMTP_FROM_ADDRESS").unwrap_or_else(|_| "noreply@solsynth.com".to_string()),
        to: vec![to_email.to_string()],
        cc: Vec::new(),
        bcc: Vec::new(),
        subject,
        html_body,
        text_body: None,
        attachments: Vec::new(),
        mail_type: MailType::EmailVerification,
        template_name: Some("email_verification".to_string()),
        template_vars: Some(serde_json::json!({
            "user_name": user_name,
            "verification_token": verification_token,
        })),
        status: MailStatus::Queued,
        retry_count: 0,
        error_message: None,
        sent_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}