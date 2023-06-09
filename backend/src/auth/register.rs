use actix_web::{post, web, HttpResponse, Responder, Result};
use harmony::Error;
use once_cell::sync::Lazy;
use pwhash::bcrypt;
use regex::Regex;
use serde::Deserialize;

static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap());
static DOB_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9]{2}-[0-9]{2}-[0-9]{4}").unwrap());

#[derive(Deserialize)]
pub struct Body {
    email: String,
    password: String,
    username: String,
    avatar: Option<String>,
    dob: String,
}

#[post("/register")]
pub async fn post(
    pool: web::Data<harmony::Pool>,
    body: web::Json<Body>,
) -> Result<impl Responder, Error> {
    // hash the password
    let hash = bcrypt::hash(body.password.as_str()).unwrap();

    // generate an id for the user
    let id = unsafe { harmony::models::user::SNOWFLAKE.generate() };

    // ensure the email is valid
    let email = body.email.clone();

    if !EMAIL_REGEX.is_match(&email) {
        Err(Error::BadClientData {
            message: "Invalid email address.".to_owned(),
        })?
    }

    // todo: username contraints

    // ensure the date of birth is valid
    let dob = body.dob.clone();

    if !DOB_REGEX.is_match(&dob) {
        Err(Error::BadClientData {
            message: "Invalid date of birth.".to_owned(),
        })?
    }

    // insert the user into the database
    let email = body.email.clone();
    let username = body.username.clone();
    let avatar = body.avatar.clone();

    harmony::get_db(&pool)
        .await
        .execute(
            "INSERT INTO users (id, username, avatar, email, password, dob) VALUES ($1, $2, $3, $4, $5, TO_DATE($6, 'DD-MM-YYYY'))",
            &[&id, &username, &avatar, &email, &hash, &dob],
        )
        .await.map_err(|err| Error::BadClientData { message: "Account already exists with that ".to_owned()
        + (if err.to_string().contains("Key (email)") {
            "email."
        } else {
            "username."}) })?;

    Ok(HttpResponse::Created().json(harmony::models::user::User {
        id,
        username,
        avatar,
        flags: 0,
    }))
}
