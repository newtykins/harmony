use actix_web::{error, HttpResponse, http::{header::ContentType, StatusCode}};
use derive_more::{Display, Error};

pub mod models;

// Sun Jan 01 2023 00:00:00 GMT+0000 (Greenwich Mean Time)
pub const SNOWFLAKE_EPOCH: i64 = 1672531200;

pub type ConnectionManager = bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>;

pub type Pool = bb8::Pool<ConnectionManager>;

#[derive(Debug, Display, Error)]
pub enum Error {
    BadClientData {
        message: String
    }
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
