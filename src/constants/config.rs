use chrono::Weekday;

pub const OPEN_AI_COMPLETIONS_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
pub const BING_SEARCH_ENDPOINT: &str = "https://api.bing.microsoft.com/v7.0/search";
pub const PERPLEXITY_SEARCH_ENDPOINT: &str = "https://api.perplexity.ai/chat/completions";

pub struct CustomEmail {
    pub topic: &'static str,
    pub subject: &'static str,
    pub schedule: &'static [Weekday],
    pub send_to: &'static str,
}

pub const CUSTOM_EMAILS: [CustomEmail; 1] = [CustomEmail {
    topic: "Retrieve the latest funding & grant programs for anything related to non-profit AI, Indigenous/Endangered languages or Australian Indigenous funding.",
    subject: "Ourland: New potential funding opportunities",
    schedule: &[Weekday::Tue, Weekday::Wed, Weekday::Fri],
    send_to: "devon@land.org.au",
}];
