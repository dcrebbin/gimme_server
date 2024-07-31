use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger::Env;

mod constants;
mod middleware;
mod models;
mod routes;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string()) // Default to 8080 if PORT is not set
        .parse::<u16>()
        .expect("PORT must be a valid number");
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::guard_middleware::ApiKeyMiddleware)
            .wrap(Logger::new("%a %{User-Agent}i %r %s %b %T")) // Single, more detailed logger
            .service(web::scope("/api").configure(|r| {
                r.route("/email", web::post().to(routes::email::send_email));
                r.route("/search", web::post().to(routes::bing::search));
                r.route("/transform", web::post().to(routes::open_ai::transform));
                r.route(
                    "/perplexity",
                    web::post().to(routes::perplexity::search_and_transform),
                );
                r.route(
                    "/notification",
                    web::post().to(routes::notification::send_notification),
                );
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
