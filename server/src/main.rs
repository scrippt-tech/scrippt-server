use actix_web::{
    middleware::{Logger, NormalizePath},
    web, App, HttpServer,
};
use dotenv::dotenv;
use env_logger::fmt::Color;
use log;
use server::handlers::account_handlers;
use server::repository::{database::DatabaseRepository, redis::RedisRepository};
use std::env;
use std::io::Write;

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
                record
                    .file()
                    .unwrap_or_default()
                    .split('/')
                    .last()
                    .unwrap_or_default(),
                record.line().unwrap_or(0),
                style.value(level),
                record.args()
            )
        })
        .init();

    log::info!("Starting server on port 8080...");

    let user = env::var("MONGO_USER").expect("MONGO_USER must be set");
    let psw = env::var("MONGO_PASSWORD").expect("MONGO_PASSWORD must be set");
    let host = env::var("MONGO_HOST").expect("MONGO_HOST must be set");
    let uri = format!(
        "mongodb+srv://{}:{}@{}/?retryWrites=true&w=majority",
        user.as_str(),
        psw.as_str(),
        host.as_str()
    );

    // Database
    let db = DatabaseRepository::new(&uri, host).await;
    let data = web::Data::new(db);

    // Redis
    let redis = RedisRepository::new("redis://127.0.0.1:6379/");
    let redis_data = web::Data::new(redis);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(NormalizePath::trim())
            .app_data(redis_data.clone())
            .app_data(data.clone())
            .service(
                web::scope("/account")
                    .service(account_handlers::get_account_by_id)
                    .service(account_handlers::create_account)
                    .service(account_handlers::authenticate_external_account)
                    .service(account_handlers::update_account)
                    .service(account_handlers::delete_account)
                    .service(account_handlers::login_account)
                    .service(account_handlers::get_verification_code),
            )
            .route("/health", web::get().to(|| async { "OK" }))
        // .service(web::scope("/api/profile").service(profile_handlers::change_profile))
        // .service(web::scope("/api/document").service(document_handlers::document))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
