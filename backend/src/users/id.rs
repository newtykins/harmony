use actix_web::{get, web, Result, HttpResponse, Responder};
use harmony::{Error};

#[get("/{id}")]
pub async fn get(pool: web::Data<harmony::Pool>, path: web::Path<i64>) -> Result<impl Responder, Error> {
	let user = harmony::get_db(&pool)
		.await
		.query_one("SELECT * FROM users WHERE id = $1", &[&path.to_owned()])
		.await
		.map_err(|_| Error::BadClientData { message: "Invalid user id.".to_string() })?;

	Ok(HttpResponse::Ok().json(harmony::models::user::User {
		id: user.get("id"),
		username: user.get("username"),
		avatar: user.get("avatar"),
		flags: user.get("flags"),
	}))
}
