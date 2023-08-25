use actix_cors::Cors;
use actix_web::{
    http,
    middleware::{Logger, NormalizePath},
    web, App, HttpServer,
};
use dotenv::dotenv;
use env_logger::fmt::Color;
use server::handlers::{account_handlers, document_handlers, generate_handlers, profile_handlers};
use server::repository::{database::DatabaseRepository, redis::RedisRepository};
use std::env;
use std::io::Write;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Check for environment variables
    env::var("MONGO_URI").expect("MONGO_URI must be set");
    env::var("REDIS_URI").expect("REDIS_URI must be set");
    env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set");
    env::var("GOOGLE_JWK_PATH").expect("GOOGLE_JWK_PATH must be set");
    env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");
    env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    env::var("ENV").expect("ENV must be set");
    env::var("APP_NAME").expect("APP_NAME must be set");
    env::var("DOMAIN").expect("DOMAIN must be set");

    // Build logger
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

    log::info!("Starting server on port 8080...");

    // Database
    let db = DatabaseRepository::new(&env::var("MONGO_URI").unwrap()).await;
    let data = web::Data::new(db);

    // Redis
    let redis = RedisRepository::new(&env::var("REDIS_URI").unwrap());
    let redis_data = web::Data::new(redis);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE])
            .max_age(3600);
        App::new()
            .wrap(cors)
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
                    .service(account_handlers::get_verification_code)
                    .service(account_handlers::verify_email),
            )
            .route("/health", web::get().to(|| async { "OK" }))
            .service(web::scope("/profile").service(profile_handlers::change_profile))
            .service(web::scope("/generate").service(generate_handlers::generate_openai))
            .service(web::scope("/document").service(document_handlers::create_update_document).service(document_handlers::delete_document))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
