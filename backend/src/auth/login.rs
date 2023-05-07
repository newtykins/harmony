use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use harmony::Error;
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

#[derive(Deserialize)]
pub struct Body {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct Response {
	token: String
}

#[post("/login")]
pub async fn post(
    pool: web::Data<harmony::Pool>,
    body: web::Json<Body>,
) -> Result<impl Responder, Error> {
	let invalid_details_error = Error::BadClientData {
		message: "Invalid email or password.".to_string(),
	};

    let user = harmony::get_db(&pool)
        .await
        .query_one(
            "SELECT id, password FROM users WHERE email = $1",
            &[&body.email],
        )
        .await
        .map_err(|_| invalid_details_error.clone())?;

	let should_authenticate = bcrypt::verify(body.password.as_str(), user.get("password"));

	if should_authenticate {
		#[derive(Serialize)]
		struct Claims {
			user_id: i64,
			expiration: i64,
		}
		let token = encode(
			&Header::new(Algorithm::HS512),
			&Claims {
				user_id: user.get("id"),
				expiration: Utc::now()
					.checked_add_signed(chrono::Duration::hours(
						dotenv!("JWT_EXPIRATION_HOURS").parse::<i64>().unwrap(),
					))
					.expect("valid timestamp")
					.timestamp(),
			},
			&EncodingKey::from_secret(dotenv!("JWT_SECRET").as_bytes()),
		).map_err(|e| Error::BadClientData { message: e.to_string() })?;

		Ok(HttpResponse::Ok().json(Response { token }))
	} else {
		Err(invalid_details_error)
	}
}
