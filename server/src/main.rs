mod models;
mod handlers;
mod repository;
mod auth;

use actix_web::{App, web, get, HttpServer, HttpResponse};
use handlers::{account_handlers, profile_handlers};
use repository::account_repository::AccountRepository;
use repository::profile_repository::ProfileRepository;
use env_logger::fmt::Color;
use std::io::Write;
use dotenv::dotenv;
use log;

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Scrippt Server")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    dotenv().ok();
    env_logger::builder()
        .format(|buf, record| {
            let level = record.level();
            let mut style = buf.style();
            match record.level() {
                log::Level::Error => style.set_color(Color::Red),
                log::Level::Warn => style.set_color(Color::Yellow),
                log::Level::Info => style.set_color(Color::Green),
                log::Level::Debug => style.set_color(Color::Blue),
                log::Level::Trace => style.set_color(Color::Cyan),
            };
            writeln!(
                buf,
                "{}:{} [{}] - {}",
                record.file().unwrap_or_default().split('/').last().unwrap_or_default(),
                record.line().unwrap_or(0),
                style.value(level),
                record.args()
            )
        })
        .init();

    // TODO: Join two repositories into one
    let acc_db = AccountRepository::new().await;
    let profile_db = ProfileRepository::new().await;
    let acc_data = web::Data::new(acc_db);
    let profile_data = web::Data::new(profile_db);
    log::info!("Server started on port 8000");

    HttpServer::new(move || {
        App::new()
        .service(index)
        .service(
            web::scope("/account")
                    .app_data(acc_data.clone())
                    .service(account_handlers::get_account_by_id)
                    .service(account_handlers::create_account)
                    .service(account_handlers::update_account)
                    .service(account_handlers::delete_account)
                    .service(account_handlers::login_account)
                )
            .service(
                web::scope("/profile")
                    .app_data(profile_data.clone())
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
