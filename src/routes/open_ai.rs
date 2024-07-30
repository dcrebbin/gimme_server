use actix_web::{web, HttpResponse};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Serialize, Deserialize)]
pub struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAiResponse {
    choices: Option<Vec<Choice>>,
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    message: Option<Message>,
}

#[derive(Deserialize, Debug)]
pub struct CompletionRequest {
    pub model: String,
    pub query: String,
}

pub async fn transform(req: web::Json<CompletionRequest>) -> HttpResponse {
    println!("Request: {:?}", req);
    let start_time: Instant = Instant::now();
    let completions_endpoint = "https://api.openai.com/v1/chat/completions";
    let api_key = match std::env::var("OPEN_AI_API_KEY") {
        Ok(key) => key,
        Err(_) => return HttpResponse::InternalServerError().body("OPEN_AI_API_KEY not set"),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();
    let openai_request = OpenAiRequest {
        model: req.model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: req.query.clone(),
            },
        ],
    };

    let response = match client
        .post(completions_endpoint)
        .headers(headers)
        .json(&openai_request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Request failed: {}", e))
        }
    };

    if !response.status().is_success() {
        return HttpResponse::InternalServerError()
            .body(format!("HTTP error! status: {}", response.status()));
    }

    let most_relevant_data: OpenAiResponse = match response.json().await {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("JSON parsing failed: {}", e))
        }
    };

    if most_relevant_data.choices.is_none()
        || most_relevant_data.choices.as_ref().unwrap().is_empty()
    {
        return HttpResponse::InternalServerError().body("No choices returned from OpenAI");
    }

    let end_time = Instant::now();
    let duration: std::time::Duration = end_time.duration_since(start_time);
    println!(
        "Transform request took: {:?} with {} and {} characters",
        duration,
        req.model,
        req.query.len()
    );

    let content = most_relevant_data.choices.unwrap()[0]
        .message
        .as_ref()
        .and_then(|m| Some(m.content.clone()));

    match content {
        Some(c) => HttpResponse::Ok().body(c),
        None => HttpResponse::InternalServerError().body("Invalid response format from OpenAI"),
    }
}
