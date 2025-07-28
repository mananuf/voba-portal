use lettre::{
    Message, SmtpTransport, Transport,
    message::{MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use tracing::{error, info};

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("SMTP configuration error: {0}")]
    Config(String),
    #[error("Email sending failed: {0}")]
    Send(#[from] lettre::transport::smtp::Error),
    #[error("Message building failed: {0}")]
    Message(#[from] lettre::error::Error),
    #[error("Address parsing failed: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("Template rendering failed: {0}")]
    Template(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub base_url: String, // For generating verification links
}

impl EmailConfig {
    pub fn from_env() -> Result<Self, EmailError> {
        Ok(Self {
            smtp_server: env::var("SMTP_SERVER")
                .map_err(|_| EmailError::Config("SMTP_SERVER not set".to_string()))?,
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .map_err(|_| EmailError::Config("Invalid SMTP_PORT".to_string()))?,
            username: env::var("SMTP_USERNAME")
                .map_err(|_| EmailError::Config("SMTP_USERNAME not set".to_string()))?,
            password: env::var("SMTP_PASSWORD")
                .map_err(|_| EmailError::Config("SMTP_PASSWORD not set".to_string()))?,
            from_email: env::var("FROM_EMAIL")
                .map_err(|_| EmailError::Config("FROM_EMAIL not set".to_string()))?,
            from_name: env::var("FROM_NAME").unwrap_or_else(|_| "Portal".to_string()),
            base_url: env::var("BASE_URL")
                .map_err(|_| EmailError::Config("BASE_URL not set".to_string()))?,
        })
    }
}

pub struct EmailService {
    mailer: SmtpTransport,
    config: EmailConfig,
}

impl EmailService {
    pub fn new() -> Result<Self, EmailError> {
        let config = EmailConfig::from_env()?;

        let creds = Credentials::new(config.username.clone(), config.password.clone());

        // Use starttls_relay instead of relay for Mailtrap compatibility
        let mailer = SmtpTransport::starttls_relay(&config.smtp_server)
            .map_err(|e| EmailError::Config(format!("SMTP starttls relay error: {}", e)))?
            .port(config.smtp_port)
            .credentials(creds)
            .build();

        Ok(Self { mailer, config })
    }

    pub fn send_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        template: EmailTemplate,
    ) -> Result<(), EmailError> {
        let to_address = match to_name {
            Some(name) => format!("{} <{}>", name, to_email),
            None => to_email.to_string(),
        };

        let from_address = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let message = if let Some(text_body) = &template.text_body {
            // Multipart email with both HTML and text
            Message::builder()
                .from(from_address.parse()?)
                .to(to_address.parse()?)
                .subject(&template.subject)
                .multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text_body.clone()),
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(template.html_body.clone()),
                        ),
                )?
        } else {
            // HTML only using SinglePart::html()
            Message::builder()
                .from(from_address.parse()?)
                .to(to_address.parse()?)
                .subject(&template.subject)
                .multipart(
                    MultiPart::alternative()
                        .singlepart(SinglePart::html(template.html_body.clone()))
                )?
        };

        info!("Sending email to: {}", to_email);
        match self.mailer.send(&message) {
            Ok(_) => {
                info!("Email sent successfully to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send email to {}: {:?}", to_email, e);
                Err(EmailError::Send(e))
            }
        }
    }

    pub fn generate_verification_template(
        &self,
        user_name: &str,
        verification_code: &str,
    ) -> EmailTemplate {
        let verification_link = format!(
            "{}/auth/verify-email?code={}",
            self.config.base_url, verification_code
        );

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Email Verification</title>
                <style>
                    body {{ 
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif; 
                        line-height: 1.6; 
                        color: #333; 
                        margin: 0; 
                        padding: 0;
                        background-color: #f5f5f5;
                    }}
                    .container {{ 
                        max-width: 600px; 
                        margin: 20px auto; 
                        background-color: #ffffff;
                        border-radius: 8px;
                        overflow: hidden;
                        box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    }}
                    .header {{ 
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white; 
                        padding: 30px 20px; 
                        text-align: center; 
                    }}
                    .header h1 {{
                        margin: 0;
                        font-size: 24px;
                        font-weight: 600;
                    }}
                    .content {{ 
                        padding: 30px 20px; 
                        background-color: #ffffff; 
                    }}
                    .content h2 {{
                        color: #333;
                        margin-top: 0;
                        font-size: 20px;
                    }}
                    .button {{ 
                        display: inline-block; 
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white !important; 
                        padding: 14px 28px; 
                        text-decoration: none; 
                        border-radius: 6px; 
                        margin: 20px 0;
                        font-weight: 600;
                        font-size: 16px;
                        transition: transform 0.2s ease;
                    }}
                    .button:hover {{
                        transform: translateY(-1px);
                    }}
                    .link-fallback {{
                        background-color: #f8f9fa;
                        padding: 15px;
                        border-radius: 6px;
                        margin: 20px 0;
                        border-left: 4px solid #667eea;
                    }}
                    .link-fallback p {{
                        margin: 0;
                        font-size: 14px;
                        color: #666;
                    }}
                    .link-fallback a {{
                        color: #667eea;
                        word-break: break-all;
                    }}
                    .footer {{ 
                        padding: 20px; 
                        text-align: center; 
                        color: #666; 
                        font-size: 14px; 
                        background-color: #f8f9fa;
                    }}
                    .warning {{
                        background-color: #fff3cd;
                        border: 1px solid #ffeaa7;
                        color: #856404;
                        padding: 12px;
                        border-radius: 6px;
                        margin: 20px 0;
                        font-size: 14px;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Welcome to VOBA 014 Portal!</h1>
                    </div>
                    <div class="content">
                        <h2>Hi {}!</h2>
                        <p>Thank you for registering with VOBA 014 Portal. To complete your registration and secure your account, please verify your email address.</p>
                        
                        <div style="text-align: center;">
                            <a href="{}" class="button">Verify Email Address</a>
                        </div>
                        
                        <div class="link-fallback">
                            <p><strong>Button not working?</strong> Copy and paste this link into your browser:</p>
                            <p><a href="{}">{}</a></p>
                        </div>
                        
                        <div class="warning">
                            <strong>⚠️ Security Notice:</strong> This verification link will expire in 24 hours for your security.
                        </div>
                        
                        <p>If you didn't create an account with VOBA 014 Portal, please ignore this email and no further action is required.</p>
                        
                        <p>Need help? Contact our support team - we're here to assist you!</p>
                    </div>
                    <div class="footer">
                        <p>&copy; 2025 VOBA 014 Portal. All rights reserved.</p>
                        <p>This is an automated message, please do not reply to this email.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            user_name, verification_link, verification_link, verification_link
        );

        let text_body = format!(
            r#"
Welcome to VOBA 014 Portal!

Hi {}!

Thank you for registering with VOBA 014 Portal. To complete your registration and secure your account, please verify your email address by visiting the following link:

{}

⚠️ SECURITY NOTICE: This verification link will expire in 24 hours for your security.

If you didn't create an account with VOBA 014 Portal, please ignore this email and no further action is required.

Need help? Contact our support team - we're here to assist you!

---
© 2025 VOBA 014 Portal. All rights reserved.
This is an automated message, please do not reply to this email.
            "#,
            user_name, verification_link
        );

        EmailTemplate {
            subject: "Verify your VOBA 014 Portal account".to_string(),
            html_body,
            text_body: Some(text_body),
        }
    }

    pub fn generate_welcome_template(&self, user_name: &str) -> EmailTemplate {
        let portal_link = format!("{}/dashboard", self.config.base_url);

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Welcome to VOBA 014 Portal</title>
                <style>
                    body {{ 
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif; 
                        line-height: 1.6; 
                        color: #333; 
                        margin: 0; 
                        padding: 0;
                        background-color: #f5f5f5;
                    }}
                    .container {{ 
                        max-width: 600px; 
                        margin: 20px auto; 
                        background-color: #ffffff;
                        border-radius: 8px;
                        overflow: hidden;
                        box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    }}
                    .header {{ 
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white; 
                        padding: 30px 20px; 
                        text-align: center; 
                    }}
                    .header h1 {{
                        margin: 0;
                        font-size: 24px;
                        font-weight: 600;
                    }}
                    .content {{ 
                        padding: 30px 20px; 
                        background-color: #ffffff; 
                    }}
                    .content h2 {{
                        color: #333;
                        margin-top: 0;
                        font-size: 20px;
                    }}
                    .success-icon {{
                        text-align: center;
                        font-size: 48px;
                        margin: 20px 0;
                    }}
                    .button {{ 
                        display: inline-block; 
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white !important; 
                        padding: 14px 28px; 
                        text-decoration: none; 
                        border-radius: 6px; 
                        margin: 20px 0;
                        font-weight: 600;
                        font-size: 16px;
                    }}
                    .features {{
                        background-color: #f8f9fa;
                        padding: 20px;
                        border-radius: 6px;
                        margin: 20px 0;
                    }}
                    .features ul {{
                        margin: 0;
                        padding-left: 20px;
                    }}
                    .features li {{
                        margin: 8px 0;
                    }}
                    .footer {{ 
                        padding: 20px; 
                        text-align: center; 
                        color: #666; 
                        font-size: 14px; 
                        background-color: #f8f9fa;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>🎉 Welcome to VOBA 014 Portal!</h1>
                    </div>
                    <div class="content">
                        <h2>Hi {}!</h2>
                        <p><strong>Congratulations!</strong> Your email has been successfully verified and your VOBA 014 Portal account is now active.</p>
                        
                        <div style="text-align: center;">
                            <a href="{}" class="button">Access Your Dashboard</a>
                        </div>
                        
                        <div class="features">
                            <h3>What you can do now:</h3>
                            <ul>
                                <li>Access your personalized dashboard</li>
                                <li>Manage your account settings</li>
                                <li>Connect with the VOBA 014 community</li>
                                <li>Explore all available features</li>
                            </ul>
                        </div>
                        
                        <p>If you have any questions or need assistance getting started, don't hesitate to reach out to our support team. We're here to help make your experience as smooth as possible!</p>
                        
                        <p>Thank you for joining VOBA 014 Portal. We're excited to have you aboard!</p>
                    </div>
                    <div class="footer">
                        <p>&copy; 2025 VOBA 014 Portal. All rights reserved.</p>
                        <p>Need help? Contact our support team anytime.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            user_name, portal_link
        );

        let text_body = format!(
            r#"
🎉 Welcome to VOBA 014 Portal!

Hi {}!

Congratulations! Your email has been successfully verified and your VOBA 014 Portal account is now active.

Access your dashboard: {}

What you can do now:
• Access your personalized dashboard
• Manage your account settings  
• Connect with the VOBA 014 community
• Explore all available features

If you have any questions or need assistance getting started, don't hesitate to reach out to our support team. We're here to help make your experience as smooth as possible!

Thank you for joining VOBA 014 Portal. We're excited to have you aboard!

---
© 2025 VOBA 014 Portal. All rights reserved.
Need help? Contact our support team anytime.
            "#,
            user_name, portal_link
        );

        EmailTemplate {
            subject: "🎉 Welcome to VOBA 014 Portal - You're all set!".to_string(),
            html_body,
            text_body: Some(text_body),
        }
    }

    /// Generate a password reset email template
    pub fn generate_password_reset_template(
        &self,
        user_name: &str,
        reset_token: &str,
    ) -> EmailTemplate {
        let reset_link = format!(
            "{}/auth/reset-password?token={}",
            self.config.base_url, reset_token
        );

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Reset Your Password</title>
                <style>
                    body {{ 
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif; 
                        line-height: 1.6; 
                        color: #333; 
                        margin: 0; 
                        padding: 0;
                        background-color: #f5f5f5;
                    }}
                    .container {{ 
                        max-width: 600px; 
                        margin: 20px auto; 
                        background-color: #ffffff;
                        border-radius: 8px;
                        overflow: hidden;
                        box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    }}
                    .header {{ 
                        background: linear-gradient(135deg, #ff6b6b 0%, #ee5a24 100%);
                        color: white; 
                        padding: 30px 20px; 
                        text-align: center; 
                    }}
                    .content {{ padding: 30px 20px; }}
                    .button {{ 
                        display: inline-block; 
                        background: linear-gradient(135deg, #ff6b6b 0%, #ee5a24 100%);
                        color: white !important; 
                        padding: 14px 28px; 
                        text-decoration: none; 
                        border-radius: 6px; 
                        margin: 20px 0;
                        font-weight: 600;
                        font-size: 16px;
                    }}
                    .warning {{
                        background-color: #fff3cd;
                        border: 1px solid #ffeaa7;
                        color: #856404;
                        padding: 12px;
                        border-radius: 6px;
                        margin: 20px 0;
                        font-size: 14px;
                    }}
                    .footer {{ 
                        padding: 20px; 
                        text-align: center; 
                        color: #666; 
                        font-size: 14px; 
                        background-color: #f8f9fa;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>🔐 Password Reset Request</h1>
                    </div>
                    <div class="content">
                        <h2>Hi {}!</h2>
                        <p>We received a request to reset your VOBA 014 Portal password. If you made this request, click the button below to set a new password:</p>
                        
                        <div style="text-align: center;">
                            <a href="{}" class="button">Reset Password</a>
                        </div>
                        
                        <div class="warning">
                            <strong>⚠️ Security Notice:</strong> This password reset link will expire in 1 hour for your security.
                        </div>
                        
                        <p>If you didn't request a password reset, please ignore this email. Your password will remain unchanged.</p>
                        
                        <p>For security reasons, this link can only be used once.</p>
                    </div>
                    <div class="footer">
                        <p>&copy; 2025 VOBA 014 Portal. All rights reserved.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            user_name, reset_link
        );

        let text_body = format!(
            r#"
🔐 Password Reset Request

Hi {}!

We received a request to reset your VOBA 014 Portal password. If you made this request, visit the following link to set a new password:

{}

⚠️ SECURITY NOTICE: This password reset link will expire in 1 hour for your security.

If you didn't request a password reset, please ignore this email. Your password will remain unchanged.

For security reasons, this link can only be used once.

---
© 2025 VOBA 014 Portal. All rights reserved.
            "#,
            user_name, reset_link
        );

        EmailTemplate {
            subject: "🔐 Reset your VOBA 014 Portal password".to_string(),
            html_body,
            text_body: Some(text_body),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_template_generation() {
        let config = EmailConfig {
            smtp_server: "sandbox.smtp.mailtrap.io".to_string(),
            smtp_port: 587,
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            from_email: "test@example.com".to_string(),
            from_name: "Test Portal".to_string(),
            base_url: "https://example.com".to_string(),
        };

        let service = EmailService {
            mailer: SmtpTransport::starttls_relay(&config.smtp_server)
                .unwrap()
                .credentials(Credentials::new(config.username.clone(), config.password.clone()))
                .build(),
            config,
        };

        let template = service.generate_verification_template("John Doe", "test123");
        assert!(template.subject.contains("VOBA 014 Portal"));
        assert!(template.html_body.contains("John Doe"));
        assert!(template.html_body.contains("test123"));
        assert!(template.text_body.is_some());
    }
}