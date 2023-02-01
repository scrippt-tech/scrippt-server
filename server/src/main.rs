mod models;
mod handlers;
mod repository;
mod auth;

use actix_web::{App, web, HttpServer, middleware::Logger};
use handlers::{account_handlers, profile_handlers, document_handlers};
use repository::db::DatabaseRepository;
use env_logger::fmt::Color;
use std::io::Write;
use dotenv::dotenv;
use log;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
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

    log::info!("Starting server on port 8000...");

    // Database
    let db = DatabaseRepository::new().await;
    let data = web::Data::new(db);

    HttpServer::new(move || {
        App::new()
        .wrap(Logger::default())
        .app_data(data.clone())
        .service(
            web::scope("/api/account")
                    .service(account_handlers::get_account_by_id)
                    .service(account_handlers::create_account)
                    .service(account_handlers::update_account)
                    .service(account_handlers::delete_account)
                    .service(account_handlers::login_account)
                )
        .service(
            web::scope("/api/profile")
                .service(profile_handlers::profile)
        )
        .service(
            web::scope("/api/document")
                .service(document_handlers::document)
        )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
