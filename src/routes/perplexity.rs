use actix_web::{web, HttpResponse};

use crate::{
    constants::{
        config::PERPLEXITY_SEARCH_ENDPOINT,
        utility::{log_error, log_query},
    },
    models::perplexity_models::{Message, PerplexityRequest, PerplexityResponse, SearchRequest},
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::time::Instant;

const SONAR_SMALL: &str = "llama-3-sonar-small-32k-online";
const SONAR_LARGE: &str = "llama-3-sonar-large-32k-online";
const PROMPT_RULES: &str =
    ". Transform the response into markdown and add the url at the end of each item.";

pub async fn search_and_transform(req: web::Json<SearchRequest>) -> HttpResponse {
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
        .post(PERPLEXITY_SEARCH_ENDPOINT)
        .headers(headers)
        .json(&perplexity_request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            log_error(&format!("Request failed: {}", e));
            return HttpResponse::InternalServerError().body(format!("Request failed: {}", e));
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
    log_query(&format!(
        "Perplexity request took: {:?} with {} and {} characters",
        duration,
        perplexity_request.model,
        perplexity_request.messages[0].content.len()
    ));

    let response_content: PerplexityResponse = match perplexity_response.json().await {
        Ok(content) => content,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("JSON parsing failed: {}", e))
        }
    };

    let content = get_perplexity_response(&response_content);
    match content {
        Some(c) => HttpResponse::Ok().body(c),
        None => {
            log_error("No content");
            HttpResponse::BadRequest().body("No content")
        }
    }
}

fn get_perplexity_response(response: &PerplexityResponse) -> Option<String> {
    response
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
}
