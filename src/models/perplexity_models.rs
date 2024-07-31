use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub use_sonar_small: Option<bool>,
}

#[derive(Serialize)]
pub struct PerplexityRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct PerplexityResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub struct Choice {
    pub message: Message,
}
