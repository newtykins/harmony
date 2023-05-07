#![feature(const_trait_impl)]

#[macro_use]
extern crate dotenvy_macro;

mod auth;

use actix_web::{
    error, get, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder,
};
use auth::auth_service;
use harmony::{ConnectionManager, Pool};
use tokio_postgres::NoTls;

#[get("/")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().json("pong")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create the database pool
    let manager = ConnectionManager::new_from_stringlike(dotenv!("DATABASE_URL"), NoTls).unwrap();

    let pool = Pool::builder()
        .build(manager)
        .await
        .expect("could not build connection pool");

    // spawn the web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // custom json extractor
            .app_data(web::JsonConfig::default().error_handler(|err, _| {
                // todo: better error handling, whatever that may entail
                let msg = err.to_string();
                error::InternalError::from_response(
                    err,
                    HttpResponse::Conflict()
                        .insert_header(ContentType::json())
                        .body(format!("{{\"message\":\"{}\"}}", msg)),
                )
                .into()
            }))
            .service(ping)
            .service(web::scope("/api/v1").service(auth_service()))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
