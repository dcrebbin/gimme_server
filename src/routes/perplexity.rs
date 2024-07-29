use actix_web::{web, HttpResponse, Responder};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Instant;

const SONAR_SMALL: &str = "llama-3-sonar-small-32k-online";
const SONAR_LARGE: &str = "llama-3-sonar-large-32k-online";
const PROMPT_RULES: &str = "";

#[derive(Deserialize)]
pub struct SearchRequest {
    query: String,
    use_sonar_small: Option<bool>,
}

#[derive(Serialize)]
struct PerplexityRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct PerplexityResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub async fn search_and_transform(req: web::Json<SearchRequest>) -> impl Responder {
    let start_time: Instant = Instant::now();
    let api_key = match std::env::var("PERPLEXITY_API_KEY") {
        Ok(key) => key,
        Err(_) => return HttpResponse::InternalServerError().body("PERPLEXITY_API_KEY not set"),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );

    let perplexity_request = PerplexityRequest {
        model: if req.use_sonar_small.unwrap_or(false) {
            SONAR_SMALL
        } else {
            SONAR_LARGE
        }
        .to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: format!("{}{}", req.query, PROMPT_RULES),
        }],
    };

    let client = reqwest::Client::new();
    let perplexity_response = match client
        .post("https://api.perplexity.ai/chat/completions")
        .headers(headers)
        .json(&perplexity_request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Request failed: {}", e))
        }
    };

    if !perplexity_response.status().is_success() {
        return HttpResponse::InternalServerError().body(format!(
            "HTTP error! status: {}",
            perplexity_response.status()
        ));
    }

    let end_time: Instant = Instant::now();
    let duration: std::time::Duration = end_time.duration_since(start_time);
    println!(
        "Perplexity request took: {:?} with {} and {} characters",
        duration,
        perplexity_request.model,
        perplexity_request.messages[0].content.len()
    );

    let response_content: PerplexityResponse = match perplexity_response.json().await {
        Ok(content) => content,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("JSON parsing failed: {}", e))
        }
    };

    let content = get_perplexity_response(&response_content);
    match content {
        Some(c) => HttpResponse::Ok().body(c),
        None => HttpResponse::BadRequest().body("No content"),
    }
}

fn get_perplexity_response(response: &PerplexityResponse) -> Option<String> {
    response
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
}
