use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    get, post, web, App, Error, HttpResponse, HttpServer, Responder,
};
use dotenv::dotenv;
use actix_web::middleware::Logger;
use env_logger::Env;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
mod routes;

struct ApiKeyMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ApiKeyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyMiddlewareService { service }))
    }
}

struct ApiKeyMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ApiKeyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let api_key = std::env::var("API_KEY").expect("API_KEY must be set");
        
        if let Some(key) = req.headers().get("x-api-key") {
            let key = key.to_str().unwrap();
            if key == api_key {
                let fut = self.service.call(req);
                return Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                });
            }
        }
        
        Box::pin(async move {
            Err(ErrorUnauthorized("Invalid API Key"))
        })
    }
}


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
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string()) // Default to 8080 if PORT is not set
        .parse::<u16>()
        .expect("PORT must be a valid number");

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(ApiKeyMiddleware)
            .wrap(Logger::new("%a %{User-Agent}i %r %s %b %T")) // Single, more detailed logger
            .service(hello)
            .service(echo)
            .route("/email", web::post().to(routes::email::send_email))
            .route("/search", web::post().to(routes::bing::search))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}