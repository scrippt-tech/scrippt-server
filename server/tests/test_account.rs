#![cfg(test)]

use actix_http::body::MessageBody;
use actix_service::ServiceFactory;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::Error,
    http,
    // middleware::Logger,
    test,
    web,
    App,
};
use env_logger;
use server::handlers::account_handlers::*;
use server::repository::database::DatabaseRepository;
// use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

async fn get_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    // set up the logger to debug
    // env::set_var("RUST_LOG", "debug");
    INIT.call_once(|| env_logger::init());
    std::env::set_var("JWT_SECRET", "secret");
    let db = DatabaseRepository::new("mongodb://localhost:27017", "localhost".to_string()).await;
    let _ = db.drop_database().await;
    App::new()
        // .wrap(Logger::default())
        .app_data(web::Data::new(db))
        .service(create_account)
        .service(get_account_by_id)
        .service(update_account)
        .service(delete_account)
        .service(login_account)
}

#[actix_rt::test]
async fn test_create_account() {
    let app = get_app().await;
    let app = test::init_service(app).await;
    let req = test::TestRequest::post()
        .uri("/create")
        .set_json(serde_json::json!({
            "name": "John Doe",
            "email": "johndoe@email.com",
            "password": "password"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("{:?}", resp.response());
    assert_eq!(resp.status(), http::StatusCode::CREATED);
}

#[actix_rt::test]
async fn test_create_account_duplicate() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = test::TestRequest::post()
        .uri("/create")
        .set_json(serde_json::json!({
            "name": "Jane Smith",
            "email": "janesmit@email.com",
            "password": "password"
        }))
        .to_request();

    let req_dup = test::TestRequest::post()
        .uri("/create")
        .set_json(serde_json::json!({
            "name": "Jane Smith",
            "email": "janesmit@email.com",
            "password": "password"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let resp = test::call_service(&server, req_dup).await;
    assert_eq!(resp.status(), 201);
}
