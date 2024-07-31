use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAiResponse {
    pub choices: Option<Vec<Choice>>,
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    pub message: Option<Message>,
}

#[derive(Deserialize, Debug)]
pub struct CompletionRequest {
    pub model: String,
    pub query: String,
}
