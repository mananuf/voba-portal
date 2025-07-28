use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    SmtpTransport, Transport, Message,
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

        let mailer = SmtpTransport::relay(&config.smtp_server)
            .map_err(|e| EmailError::Config(format!("SMTP relay error: {}", e)))?
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

        let mut message_builder = Message::builder()
            .from(from_address.parse()?)
            .to(to_address.parse()?)
            .subject(&template.subject);

        let message = if let Some(text_body) = &template.text_body {
            // Multipart email with both HTML and text
            message_builder.multipart(
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
            // HTML only
            message_builder
                .header(ContentType::TEXT_HTML)
                .body(template.html_body.clone())?
        };

        info!("Sending email to: {}", to_email);
        self.mailer.send(&message)?;
        info!("Email sent successfully to: {}", to_email);

        Ok(())
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
            <html>
            <head>
                <meta charset="utf-8">
                <title>Email Verification</title>
                <style>
                    body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background-color: #4CAF50; color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 20px; background-color: #f9f9f9; }}
                    .button {{ 
                        display: inline-block; 
                        background-color: #4CAF50; 
                        color: white; 
                        padding: 12px 24px; 
                        text-decoration: none; 
                        border-radius: 5px; 
                        margin: 20px 0;
                    }}
                    .footer {{ padding: 20px; text-align: center; color: #666; font-size: 12px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Welcome to VOBA 014 Portal!</h1>
                    </div>
                    <div class="content">
                        <h2>Hi {}!</h2>
                        <p>Thank you for registering with VOBA 014 Portal. To complete your registration, please verify your email address by clicking the button below:</p>
                        
                        <div style="text-align: center;">
                            <a href="{}" class="button">Verify Email Address</a>
                        </div>
                        
                        <p>If the button above doesn't work, you can also copy and paste the following link into your browser:</p>
                        <p style="word-break: break-all;"><a href="{}">{}</a></p>
                        
                        <p>This verification link will expire in 24 hours for security reasons.</p>
                        
                        <p>If you didn't create an account with Portal, please ignore this email.</p>
                    </div>
                    <div class="footer">
                        <p>&copy; 2025 Portal. All rights reserved.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            user_name, verification_link, verification_link, verification_link
        );

        let text_body = format!(
            r#"
Welcome to Portal!

Hi {}!

Thank you for registering with Portal. To complete your registration, please verify your email address by visiting the following link:

{}

This verification link will expire in 24 hours for security reasons.

If you didn't create an account with Portal, please ignore this email.

© 2025 Portal. All rights reserved.
            "#,
            user_name, verification_link
        );

        EmailTemplate {
            subject: "Verify your Portal account".to_string(),
            html_body,
            text_body: Some(text_body),
        }
    }

    pub fn generate_welcome_template(&self, user_name: &str) -> EmailTemplate {
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Welcome to Portal</title>
                <style>
                    body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background-color: #4CAF50; color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 20px; background-color: #f9f9f9; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Welcome to Portal!</h1>
                    </div>
                    <div class="content">
                        <h2>Hi {}!</h2>
                        <p>Your email has been successfully verified. Welcome to Portal!</p>
                        <p>You can now access all features of your account.</p>
                        <p>If you have any questions, feel free to contact our support team.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            user_name
        );

        EmailTemplate {
            subject: "Welcome to Portal - Email Verified!".to_string(),
            html_body,
            text_body: Some(format!("Hi {}!\n\nYour email has been successfully verified. Welcome to Portal!\n\nYou can now access all features of your account.", user_name)),
        }
    }
}