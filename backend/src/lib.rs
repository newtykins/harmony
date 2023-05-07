use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use bb8_postgres::PostgresConnectionManager;
use derive_more::{Display, Error};
use tokio_postgres::NoTls;

pub mod models;

// Sun Jan 01 2023 00:00:00 GMT+0000 (Greenwich Mean Time)
pub const SNOWFLAKE_EPOCH: i64 = 1672531200;

pub type ConnectionManager = bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>;

pub type Pool = bb8::Pool<ConnectionManager>;

pub async fn get_db<'t>(
    pool: &'t Pool,
) -> bb8::PooledConnection<'t, PostgresConnectionManager<NoTls>> {
    pool.get()
        .await
        .expect("couldn't get db connection from pool")
}

#[derive(Debug, Display, Error, Clone)]
pub enum Error {
    BadClientData { message: String },
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(format!("{{\"message\":\"{}\"}}", self.to_string()))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Error::BadClientData { .. } => StatusCode::BAD_REQUEST,
        }
    }
}
