use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::time::Duration;
use tracing::{error, info};

const FORGET_PASSWORD_TEXT_TEMPLATE: &str = r#"
[Runtime.land] ( https://runtime.land )

************
Hi {{email}},
************

You recently requested to reset your password for your account. Use the button below to reset it. This password reset is only valid for the next 24 hours.

Reset your password ( {{reset_url}} )

If you did not request a password reset, please ignore this email or contact support ( https://runtime.land ) if you have questions.

Thanks,
The Runtime.land team
"#;

pub async fn send_forget_password_email(to: String, link: String) {
    let smtp = land_dao::settings::get_email_setting().await;
    let from = "fuxiaohei@vip.qq.com".to_string();
    let content = FORGET_PASSWORD_TEXT_TEMPLATE
        .replace("{{email}}", &to)
        .replace("{{reset_url}}", &link);
    let email = Message::builder()
        .from(smtp.from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject("Forgot Password: Email Reset")
        .header(ContentType::TEXT_PLAIN)
        .body(content)
        .unwrap();

    let creds = Credentials::new(smtp.username, smtp.password);

    let mailer = SmtpTransport::relay(&smtp.host)
        .unwrap()
        .port(smtp.port.parse().unwrap())
        .timeout(Some(Duration::from_secs(10)))
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => info!(
            "Email sent successfully!, from:{}, to:{}, link:{}",
            from, to, link
        ),
        Err(e) => error!("Could not send email: {e:?}"),
    }
}
