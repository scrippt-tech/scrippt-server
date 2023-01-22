use actix_web::{get, post, delete, put, Responder};

#[get("")]
pub async fn get_account() -> impl Responder {
    "Hello world!"
}

#[post("")]
pub async fn create_account() -> impl Responder {
    "Hello there!"
}

#[get("/{id}")]
pub async fn get_account_by_id() -> impl Responder {
    "Hello there!"
}


#[put("/{id}")]
pub async fn update_account() -> impl Responder {
    "Hello there!"
}

#[delete("/{id}")]
pub async fn delete_account() -> impl Responder {
    "Hello there!"
}
