use actix_web::{body::MessageBody, web, Error};
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
                println!("Search result: {:?}", search_result);
                converted_markdowns.push(search_result);
            }

            let combined_markdown = converted_markdowns
                .iter()
                .map(|markdown| markdown.to_string())
                .collect::<Vec<String>>()
                .join("\n");

            let transformed_markdown = open_ai::transform(web::Json(CompletionRequest {
                model: "gpt-4o".to_string(),
                query:
                    "Convert this text into markdown so it's 100% valid and using the correct markdown formatting, replace all placeholder content with the content from Input. Remove any irrelevant content. Only return the formatted markdown response with no code blocks or anything else. Example Template:".to_string() +
                    "### {Title}

### {Section 1}

**{Item 1}**
  - **{Sub Heading}:** {content}
  - **{Description}:** {content}
  - [Learn more]({url})

**{Item 2}**
  - **{Sub Heading}** {content}
  - **{Description}:** {content}
  - [Learn more]({url})
  
### {Section 2}

**{Item 1}**
  - **{Sub Heading}:** {content}
  - **{Description}:** {content}
  - [Learn more]({url})

**{Item 2}**
  - **{Sub Heading}:** {content}
  - **{Description}:** {content}
  - [Learn more]({url})"
                        + " Input:"+&combined_markdown,
            }))
            .await;

            let transformed_markdown = transformed_markdown.into_body();
            let transformed_markdown =
                String::from_utf8(transformed_markdown.try_into_bytes().unwrap().to_vec()).unwrap();

            fs::write("converted_markdown.md", transformed_markdown.clone()).unwrap();

            println!("Converted HTML: {:?}", transformed_markdown);
            let converted_html = markdown_to_html(&transformed_markdown);

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
