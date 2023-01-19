use actix_web::{web, App, HttpServer};
mod handlers;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");

    HttpServer::new(|| {
        App::new()
            .service(handlers::index)
            .service(handlers::hello)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
