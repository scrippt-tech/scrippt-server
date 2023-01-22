use actix_web::{App, web, HttpServer};
mod account_handlers;
mod profile_handlers;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(|| async { "Hello world!!" }))
            .service(
                web::scope("/account")
                    .service(account_handlers::get_account)
                    .service(account_handlers::get_account_by_id)
                    .service(account_handlers::create_account)
                    .service(account_handlers::update_account)
                    .service(account_handlers::delete_account)
                )
            .service(
                web::scope("/profile")
                    .service(profile_handlers::create_profile)
                    .service(profile_handlers::get_profile_by_id)
                    .service(profile_handlers::update_profile)
                    .service(profile_handlers::delete_profile)
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
