use crate::{
    constants::config::BING_SEARCH_ENDPOINT,
    constants::utility::{log_error, log_query},
    models::bing_models::{BingSearchResponse, SearchQuery, WebPage},
};
use actix_web::{web, HttpResponse};
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::Instant;
pub async fn search(request: web::Json<SearchQuery>) -> HttpResponse {
    let start_time: Instant = Instant::now();
    let search_endpoint = BING_SEARCH_ENDPOINT;
    let api_key = match std::env::var("BING_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            log_error("BING_API_KEY not set");
            return HttpResponse::InternalServerError().body("BING_API_KEY not set");
        }
    };

    let mut headers = HeaderMap::new();
    match HeaderValue::from_str(&api_key) {
        Ok(value) => headers.insert("Ocp-Apim-Subscription-Key", value),
        Err(_) => {
            log_error("Invalid API key format");
            return HttpResponse::InternalServerError().body("Invalid API key format");
        }
    };

    use url::form_urlencoded;

    let client = reqwest::Client::new();
    let encoded_query = form_urlencoded::Serializer::new(String::new())
        .append_pair("q", request.query.as_str())
        .finish();

    let search_response = match client
        .get(&format!("{}?{}", search_endpoint, encoded_query))
        .headers(headers)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            log_error(&format!("Request failed: {}", e));
            return HttpResponse::InternalServerError().body(format!("Request failed: {}", e));
        }
    };

    if !search_response.status().is_success() {
        log_error(&format!("HTTP error! status: {}", search_response.status()));
        return HttpResponse::InternalServerError()
            .body(format!("HTTP error! status: {}", search_response.status()));
    }

    let response = match search_response.json::<BingSearchResponse>().await {
        Ok(text) => text,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to get response text: {}", e))
        }
    };
    let end_time: Instant = Instant::now();
    let duration: std::time::Duration = end_time.duration_since(start_time);
    log_query(&format!("Bing request took: {:?}", duration));

    match get_bing_search_data(&response) {
        Some(search_data) => HttpResponse::Ok().json(search_data),
        None => HttpResponse::InternalServerError().body("JSON parsing failed"),
    }
}

fn get_bing_search_data(response_text: &BingSearchResponse) -> Option<Vec<WebPage>> {
    let content = response_text.web_pages.value.clone();
    return Some(content);
}
