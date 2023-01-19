use actix_web::Responder;

pub async fn index() -> impl Responder {
    "Hello world!"
}

pub async fn hello() -> impl Responder {
    "Hello there!"
}