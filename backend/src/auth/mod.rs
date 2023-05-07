mod register;

pub fn auth_service() -> actix_web::Scope {
    actix_web::web::scope("/auth").service(register::post)
}
