use actix_web::{ get, post, web, App, HttpResponse, HttpServer, Responder };
use dotenv::dotenv;
use actix_web::middleware::Logger;

mod routes;
use env_logger::Env;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = std::env::var("PORT").unwrap();
    let port = port.parse::<u16>().unwrap();

    env_logger::init_from_env(Env::default().default_filter_or("info"));


    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(hello)
            .service(echo)
            .route("/email", web::post().to(routes::email::send_email))
            .route("/hey", web::get().to(manual_hello))
        })
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
