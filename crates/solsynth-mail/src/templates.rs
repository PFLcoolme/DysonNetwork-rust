//! 邮件模板引擎
//!
//! 提供模板渲染功能

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

use crate::models::MailTemplate;

/// 模板管理器
#[derive(Clone)]
pub struct TemplateManager {
    templates: HashMap<String, MailTemplate>,
}

impl TemplateManager {
    /// 创建新的模板管理器
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// 注册模板
    pub fn register_template(&mut self, template: MailTemplate) {
        info!("注册模板: {}", template.name);
        self.templates.insert(template.name.clone(), template);
    }

    /// 获取模板
    pub fn get_template(&self, name: &str) -> Option<&MailTemplate> {
        self.templates.get(name)
    }

    /// 渲染模板
    pub fn render_template(
        &self,
        template_name: &str,
        variables: &Value,
    ) -> Result<(String, String)> {
        let template = self
            .templates
            .get(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_name))?;

        let subject = self.render_string(&template.subject, variables)?;
        let html_body = self.render_string(&template.html_body, variables)?;

        Ok((subject, html_body))
    }

    /// 渲染字符串模板
    fn render_string(&self, template: &str, variables: &Value) -> Result<String> {
        let mut result = template.to_string();

        if let Value::Object(map) = variables {
            for (key, value) in map {
                let placeholder = format!("{{{{{}}}}}", key);
                result = result.replace(&placeholder, &value.to_string());
            }
        }

        Ok(result)
    }

    /// 注册内置模板
    pub fn register_builtin_templates(&mut self) {
        // 密码重置模板
        self.templates.insert(
            "password_reset".to_string(),
            MailTemplate {
                id: uuid::Uuid::new_v4(),
                name: "password_reset".to_string(),
                subject: "重置您的密码".to_string(),
                html_body: r#"
<html>
    <body>
        <h1>重置密码</h1>
        <p>您好 {{user_name}},</p>
        <p>我们收到了重置密码的请求。点击以下链接重置您的密码：</p>
        <a href="{{reset_url}}" style="background-color: #4CAF50; color: white; padding: 14px 20px; text-decoration: none; border-radius: 4px;">重置密码</a>
        <p>或者使用以下代码：{{verification_code}}</p>
        <p>此链接将在 30 分钟后过期。</p>
        <p>如果您没有请求重置密码，请忽略此邮件。</p>
        <br>
        <p>Solsynth 团队</p>
    </body>
</html>
                "#
                .to_string(),
                text_body: Some("您好 {{user_name}},\n\n我们收到了重置密码的请求。请访问: {{reset_url}}\n或使用代码: {{verification_code}}\n\n此链接将在 30 分钟后过期。\n\nSolsynth 团队".to_string()),
                variables: vec![
                    "user_name".to_string(),
                    "reset_url".to_string(),
                    "verification_code".to_string(),
                ],
            },
        );

        // 邮箱验证模板
        self.templates.insert(
            "email_verification".to_string(),
            MailTemplate {
                id: uuid::Uuid::new_v4(),
                name: "email_verification".to_string(),
                subject: "验证您的邮箱".to_string(),
                html_body: r#"
<html>
    <body>
        <h1>验证邮箱</h1>
        <p>您好 {{user_name}},</p>
        <p>感谢您的注册！请点击以下链接验证您的邮箱：</p>
        <a href="{{verification_url}}" style="background-color: #2196F3; color: white; padding: 14px 20px; text-decoration: none; border-radius: 4px;">验证邮箱</a>
        <p>或者使用以下代码：{{verification_code}}</p>
        <p>此链接将在 24 小时后过期。</p>
        <br>
        <p>Solsynth 团队</p>
    </body>
</html>
                "#
                .to_string(),
                text_body: Some("您好 {{user_name}},\n\n感谢您的注册！请访问: {{verification_url}}\n或使用代码: {{verification_code}}\n\n此链接将在 24 小时后过期。\n\nSolsynth 团队".to_string()),
                variables: vec![
                    "user_name".to_string(),
                    "verification_url".to_string(),
                    "verification_code".to_string(),
                ],
            },
        );

        // 通知模板
        self.templates.insert(
            "notification".to_string(),
            MailTemplate {
                id: uuid::Uuid::new_v4(),
                name: "notification".to_string(),
                subject: "{{subject}}".to_string(),
                html_body: r#"
<html>
    <body>
        <h1>{{subject}}</h1>
        <p>您好 {{user_name}},</p>
        <p>{{message}}</p>
        <br>
        <p>Solsynth 团队</p>
    </body>
</html>
                "#
                .to_string(),
                text_body: Some("您好 {{user_name}},\n\n{{subject}}\n\n{{message}}\n\nSolsynth 团队".to_string()),
                variables: vec![
                    "user_name".to_string(),
                    "subject".to_string(),
                    "message".to_string(),
                ],
            },
        );
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        let mut tm = Self::new();
        tm.register_builtin_templates();
        tm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let manager = TemplateManager::new();
        let template = MailTemplate {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            subject: "Hello {{name}}".to_string(),
            html_body: "<p>Hello {{name}}, welcome to {{platform}}!</p>".to_string(),
            text_body: Some("Hello {{name}}, welcome to {{platform}}!".to_string()),
            variables: vec!["name".to_string(), "platform".to_string()],
        };

        let variables = serde_json::json!({
            "name": "John",
            "platform": "Solsynth"
        });

        let rendered = manager.render_string(&template.html_body, &variables).unwrap();
        assert!(rendered.contains("John"));
        assert!(rendered.contains("Solsynth"));
    }
}