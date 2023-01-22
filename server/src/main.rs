use actix_web::{App, web, get, HttpServer};
mod account_handlers;
mod profile_handlers;
mod db;

#[get("/")]
async fn index() -> &'static str {
    "Scrippt server"
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");

    let _db = db::init_db().await;

    HttpServer::new(|| {
        App::new()
            .service(index)
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
