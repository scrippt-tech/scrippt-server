mod models;
mod handlers;
mod repository;
mod auth;
mod routes;

use actix_web::{App, web, HttpServer};
use handlers::{account_handlers, profile_handlers};
use repository::db::DatabaseRepository;
use handlebars::Handlebars;
use env_logger::fmt::Color;
use std::io::Write;
use dotenv::dotenv;
use log;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Logger
    std::env::set_var("RUST_LOG", "info");
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

    // Database
    let db = DatabaseRepository::new().await;
    let data = web::Data::new(db);

    // Handlebars
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./static/templates")
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    log::info!("Server started on port 8000");

    HttpServer::new(move || {
        App::new()
        .app_data(data.clone())
        .app_data(handlebars_ref.clone())
        .service(routes::index)
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
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
