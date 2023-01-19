use actix_web::{get, post, Responder};

#[get("/")]
pub async fn index() -> impl Responder {
    "Hello world!"
}

#[post("/hello")]
pub async fn hello() -> impl Responder {
    "Hello there!"
}