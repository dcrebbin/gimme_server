use crate::{
    constants::{
        config::CUSTOM_EMAILS,
        utility::{is_development, log_query},
    },
    models::{
        bing_models::SearchQuery, open_ai_models::CompletionRequest,
        perplexity_models::SearchRequest,
    },
    routes::{
        bing::search,
        email::{send_email, Email},
        open_ai, perplexity,
    },
};
use actix_web::{body::MessageBody, web, Error};
use chrono::Datelike;

use pulldown_cmark::{html, Options, Parser};
use std::{fs, time::Instant};

pub const MARKDOWN_TEMPLATE: &str = include_str!("../templates/markdown_template.md");
pub const SEARCH_OPTIMISATION_PROMPT: &str = "Optimise this natural language query to show the best and latest results in a search engine. Only return the updated query. If the query contains more than 1 request then split it into multiple queries using semi-colons ;. Query:";

pub async fn send_notification() -> Result<String, Error> {
    let use_open_ai: bool = std::env::var("USE_OPEN_AI").unwrap() == "true";

    for email in CUSTOM_EMAILS {
        if email.schedule.contains(&chrono::Local::now().weekday()) {
            let start_time = Instant::now();
            let search_results: Vec<String> = create_optimized_search_queries(&email.topic).await;

            let mut converted_markdowns = Vec::new();
            for search_result in search_results {
                let search_result = if use_open_ai {
                    open_ai_search_and_transform(&search_result).await
                } else {
                    perplexity_search_and_transform(&search_result).await
                };
                converted_markdowns.push(search_result);
            }

            let combined_results = converted_markdowns
                .iter()
                .map(|markdown| markdown.to_string())
                .collect::<Vec<String>>()
                .join("\n");

            let converted_markdown = convert_to_markdown(&combined_results).await;
            let converted_html = markdown_to_html(&converted_markdown);

            if is_development() {
                log_query(&format!("Converted HTML: {:?}", converted_html));
                fs::write("converted_template.html", converted_html.clone()).unwrap();
            }
            let duration = start_time.elapsed();
            log_query(&format!("Notification took: {:?}", duration));

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

async fn create_optimized_search_queries(topic: &str) -> Vec<String> {
    let search_responder = open_ai::transform(web::Json(CompletionRequest {
        model: "gpt-4o-mini".to_string(),
        query: SEARCH_OPTIMISATION_PROMPT.to_string() + topic,
    }))
    .await;

    let response_body = search_responder.into_body();

    let search_optimised_query =
        String::from_utf8(response_body.try_into_bytes().unwrap().to_vec()).unwrap();

    search_optimised_query
        .split(";")
        .map(|s| s.to_string())
        .collect()
}

pub async fn open_ai_search_and_transform(query: &str) -> String {
    let json_query = web::Json(SearchQuery {
        query: query.to_string(),
    });

    let search_results = search(json_query).await;

    let search_results = search_results.into_body();

    let stringified_search_results =
        String::from_utf8(search_results.try_into_bytes().unwrap().to_vec()).unwrap();

    let transformed_search_results = open_ai::transform(web::Json(CompletionRequest {
        model: "gpt-4o-mini".to_string(),
        query: "Retrieve the most relevant information from the following search results and return it in markdown format. If there are no results then return nothing. Use the following markdown template".to_string() + MARKDOWN_TEMPLATE + " Input:" + &stringified_search_results,
    }))
    .await;

    let transformed_search_results = transformed_search_results.into_body();
    let transformed_search_results = String::from_utf8(
        transformed_search_results
            .try_into_bytes()
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    transformed_search_results
}

pub async fn perplexity_search_and_transform(query: &str) -> String {
    let search_result = perplexity::search_and_transform(web::Json(SearchRequest {
        query: query.to_string(),
        use_sonar_small: Some(false),
    }))
    .await;

    let search_result = search_result.into_body();
    let search_result =
        String::from_utf8(search_result.try_into_bytes().unwrap().to_vec()).unwrap();

    search_result
}

pub async fn convert_to_markdown(markdown: &str) -> String {
    let start_time = Instant::now();
    let transformed_markdown = open_ai::transform(web::Json(CompletionRequest {
        model: "gpt-4o-mini".to_string(),
        query:
            "Convert this text into markdown so it's 100% valid and using the correct markdown formatting, replace all placeholder content with the content from Input. Remove any irrelevant content. Only return the formatted markdown response with no code blocks or anything else. Example Template:".to_string() +
            MARKDOWN_TEMPLATE +
            " Input:" +
            &markdown,
    }))
    .await;

    let transformed_markdown = transformed_markdown.into_body();
    let transformed_markdown =
        String::from_utf8(transformed_markdown.try_into_bytes().unwrap().to_vec()).unwrap();

    if is_development() {
        fs::write("converted_markdown.md", transformed_markdown.clone()).unwrap();
        log_query(&format!("Converted HTML: {:?}", transformed_markdown));
    }
    let duration = start_time.elapsed();
    log_query(&format!("Markdown conversion took: {:?}", duration));
    transformed_markdown
}

pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
