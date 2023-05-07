mod id;

pub fn users_service() -> actix_web::Scope {
	actix_web::web::scope("/users").service(id::get)
}
