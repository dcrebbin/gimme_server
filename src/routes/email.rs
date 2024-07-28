use actix_web::{ web, Error };
use serde::{ Deserialize };

#[derive(Deserialize)]
pub struct Email {
    email: String,
    subject: String,
    body: String,
}

pub async fn send_email(info: web::Json<Email>) -> Result<String, Error> {    
    println!("Email: {}, Subject: {}, Body: {}", info.email, info.subject, info.body);
    Ok(format!("Email sent!"))
}
