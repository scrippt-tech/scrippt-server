use actix_web::{get, post, delete, put, Responder};

#[get("")]
pub async fn get_users() -> impl Responder {
    "Hello world!"
}

#[post("")]
pub async fn create_user() -> impl Responder {
    "Hello there!"
}

#[get("/{id}")]
pub async fn get_user_by_id() -> impl Responder {
    "Hello there!"
}


#[put("/{id}")]
pub async fn update_user() -> impl Responder {
    "Hello there!"
}

#[delete("/{id}")]
pub async fn delete_user() -> impl Responder {
    "Hello there!"
}
