use actix_web::{body::MessageBody, web, Error, HttpRequest, Responder};
use chrono::{Datelike, Weekday};
use pulldown_cmark::{html, Options, Parser};
use std::{fs, time::Instant};

use crate::routes::{
    email::{send_email, Email},
    open_ai::{self, CompletionRequest},
    perplexity::{self, SearchRequest},
};

#[derive(Clone)]
pub struct CustomEmail {
    topic: &'static str,
    subject: &'static str,
    schedule: &'static [Weekday],
    send_to: &'static str,
}
pub const CUSTOM_EMAILS: [CustomEmail; 1] = [CustomEmail {
    topic: "Retrieve the latest funding & grant programs for anything related to non-profit AI, Indigenous/Endangered languages or Australian Indigenous funding.",
    subject: "Ourland: New potential funding opportunities",
    schedule: &[Weekday::Tue, Weekday::Wed, Weekday::Fri],
    send_to: "devon@land.org.au",
}];

const SEARCH_OPTIMISATION_PROMPT: &str = "Optimise this natural language query to show the best and latest results in a search engine. Only return the updated query. If the query contains more than 1 request then split it into multiple queries using semi-colons ;. Query:";
const MARKDOWN_PROMPT: &str =
    "Transform this natural language query into markdown and only return the response. Query:";

pub async fn send_notification() -> Result<String, Error> {
    for email in CUSTOM_EMAILS {
        if email.schedule.contains(&chrono::Local::now().weekday()) {
            let start_time = Instant::now();

            let search_responder = open_ai::transform(web::Json(CompletionRequest {
                model: "gpt-4o-mini".to_string(),
                query: SEARCH_OPTIMISATION_PROMPT.to_string() + &email.topic.to_string(),
            }))
            .await;

            let response_body = search_responder.into_body();

            let search_optimised_query =
                String::from_utf8(response_body.try_into_bytes().unwrap().to_vec()).unwrap();

            let search_results = search_optimised_query.split(";");

            let mut converted_markdowns = Vec::new();
            for search_result in search_results {
                let search_result = perplexity::search_and_transform(web::Json(SearchRequest {
                    query: search_result.to_string(),
                    use_sonar_small: Some(false),
                }))
                .await;

                let search_result = search_result.into_body();
                let search_result =
                    String::from_utf8(search_result.try_into_bytes().unwrap().to_vec()).unwrap();

                let converted_markdown_response =
                    open_ai::transform(web::Json(CompletionRequest {
                        query: MARKDOWN_PROMPT.to_string() + &search_result,
                        model: "gpt-4o-mini".to_string(),
                    }))
                    .await;
                let converted_markdown = converted_markdown_response.into_body();
                let converted_markdown =
                    String::from_utf8(converted_markdown.try_into_bytes().unwrap().to_vec())
                        .unwrap();
                converted_markdowns.push(converted_markdown);
            }

            let converted_html = converted_markdowns
                .iter()
                .map(|markdown| markdown_to_html(&markdown))
                .collect::<Vec<String>>()
                .join("\n");

            println!("Converted HTML: {:?}", converted_html);
            fs::write("converted_html.html", converted_html.clone()).unwrap();
            let duration = start_time.elapsed();
            println!("Notification took: {:?}", duration);

            let _ = send_email(web::Json(Email {
                email: email.send_to.to_string(),
                subject: email.subject.to_string(),
                body: converted_html.clone(),
            }))
            .await;
        }
    }

    Ok(format!("Notification/s sent!"))
}

pub fn markdown_to_html(markdown: &str) -> String {
    // Set up options (you can customize these)
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    // Parse the markdown
    let parser = Parser::new_ext(markdown, options);

    // Write to String buffer
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}
