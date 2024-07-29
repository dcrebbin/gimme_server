use std::time::Instant;
use actix_web::{web, Error};
use dotenv::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::SmtpTransport;
use lettre::{Message, Transport};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Email {
    email: String,
    subject: String,
    body: String,
}

pub async fn send_email(info: web::Json<Email>) -> Result<String, Error> {
    dotenv().ok();

    let start_time: Instant = Instant::now();

    let smtp_host = std::env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let smtp_username = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");

    let email = Message::builder()
        .from(smtp_username.parse().unwrap())
        .to(info.email.parse().unwrap())
        .subject(&info.subject)
        .body(info.body.clone())
        .unwrap();

    let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
    let mailer = SmtpTransport::relay(&smtp_host)
        .unwrap()
        .credentials(creds)
        .build();
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => eprintln!("Could not send email: {:?}", e),
    }

    let end_time: Instant = Instant::now();
    let duration: std::time::Duration = end_time.duration_since(start_time);
    println!("Email request took: {:?}", duration);

    Ok(format!("Email sent!"))
}
