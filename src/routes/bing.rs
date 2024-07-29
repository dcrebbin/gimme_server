use actix_web::{web, HttpResponse, Responder};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Deserialize, Serialize)]
pub struct BingSearchResponse {
    pub _type: String,
    #[serde(rename = "queryContext")]
    pub query_context: QueryContext,
    #[serde(rename = "webPages")]
    pub web_pages: WebPages,
    #[serde(rename = "rankingResponse")]
    pub ranking_response: RankingResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryContext {
    #[serde(rename = "originalQuery")]
    pub original_query: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebPages {
    #[serde(rename = "webSearchUrl")]
    pub web_search_url: String,
    #[serde(rename = "totalEstimatedMatches")]
    pub total_estimated_matches: u64,
    pub value: Vec<WebPage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebPage {
    pub id: String,
    #[serde(rename = "contractualRules")]
    pub contractual_rules: Option<Vec<ContractualRule>>,
    pub name: String,
    pub url: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,
    #[serde(rename = "isFamilyFriendly")]
    pub is_family_friendly: bool,
    #[serde(rename = "displayUrl")]
    pub display_url: String,
    pub snippet: String,
    #[serde(rename = "dateLastCrawled")]
    pub date_last_crawled: String,
    #[serde(rename = "primaryImageOfPage")]
    pub primary_image_of_page: Option<PrimaryImageOfPage>,
    #[serde(rename = "cachedPageUrl")]
    pub cached_page_url: String,
    pub language: String,
    #[serde(rename = "isNavigational")]
    pub is_navigational: bool,
    #[serde(rename = "richFacts")]
    pub rich_facts: Option<Vec<RichFact>>,
    #[serde(rename = "noCache")]
    pub no_cache: bool,
    #[serde(rename = "siteName")]
    pub site_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractualRule {
    #[serde(rename = "targetPropertyName")]
    pub target_property_name: String,
    #[serde(rename = "targetPropertyIndex")]
    pub target_property_index: Option<u32>,
    #[serde(rename = "mustBeCloseToContent")]
    pub must_be_close_to_content: bool,
    #[serde(rename = "license")]
    pub license: License,
    #[serde(rename = "licenseNotice")]
    pub license_notice: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct License {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PrimaryImageOfPage {
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "imageId")]
    pub image_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RichFact {
    pub label: Label,
    pub items: Vec<Item>,
    pub hint: Hint,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Label {
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Item {
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Hint {
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RankingResponse {
    pub mainline: Mainline,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Mainline {
    pub items: Vec<RankingItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RankingItem {
    #[serde(rename = "answerType")]
    pub answer_type: String,
    #[serde(rename = "resultIndex")]
    pub result_index: u32,
    pub value: RankingItemValue,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RankingItemValue {
    pub id: String,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub async fn search(query: web::Json<SearchQuery>) -> impl Responder {
    let start_time: Instant = Instant::now();
    let search_endpoint = "https://api.bing.microsoft.com/v7.0/search";
    let api_key = match std::env::var("BING_API_KEY") {
        Ok(key) => key,
        Err(_) => return HttpResponse::InternalServerError().body("BING_API_KEY not set"),
    };

    println!("{:?}", query.q);

    let mut headers = HeaderMap::new();
    match HeaderValue::from_str(&api_key) {
        Ok(value) => headers.insert("Ocp-Apim-Subscription-Key", value),
        Err(_) => return HttpResponse::InternalServerError().body("Invalid API key format"),
    };

    use url::form_urlencoded;

    let client = reqwest::Client::new();
    let encoded_query = form_urlencoded::Serializer::new(String::new())
        .append_pair("q", query.q.as_str())
        .finish();

    let search_response = match client
        .get(&format!("{}?{}", search_endpoint, encoded_query))
        .headers(headers)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Request failed: {}", e);
            return HttpResponse::InternalServerError().body(format!("Request failed: {}", e));
        }
    };

    if !search_response.status().is_success() {
        return HttpResponse::InternalServerError()
            .body(format!("HTTP error! status: {}", search_response.status()));
    }
    println!("{:?}", search_response.status());

    // Print the raw JSON response
    let response = match search_response.json::<BingSearchResponse>().await {
        Ok(text) => text,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to get response text: {}", e))
        }
    };
    let end_time: Instant = Instant::now();
    let duration: std::time::Duration = end_time.duration_since(start_time);
    println!("Bing request took: {:?}", duration);

    match get_bing_search_data(&response) {
        Some(search_data) => HttpResponse::Ok().json(search_data),
        None => HttpResponse::InternalServerError().body("JSON parsing failed"),
    }
}

fn get_bing_search_data(response_text: &BingSearchResponse) -> Option<Vec<WebPage>> {
    let content = response_text.web_pages.value.clone();
    return Some(content);
}
