#![cfg(test)]

use actix_http::{body::MessageBody, header};
use actix_service::ServiceFactory;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::Error,
    middleware,
    // middleware::Logger,
    test,
    web,
    App,
};
use env_logger;
use server::handlers::account_handlers::*;
use server::repository::database::DatabaseRepository;
// use std::env;
use server::auth::jwt::decode_jwt;
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
    INIT.call_once(|| env_logger::init());
    let db = DatabaseRepository::new("mongodb://localhost:27017", "localhost".to_string()).await;
    let _ = db.drop_database().await;
    App::new()
        .wrap(middleware::Logger::default())
        .app_data(web::Data::new(db))
        .service(create_account)
        .service(get_account_by_id)
        .service(update_account)
        .service(delete_account)
        .service(login_account)
}

fn create_john_doe() -> actix_http::Request {
    test::TestRequest::post()
        .uri("/create")
        .set_json(serde_json::json!({
            "name": "John Doe",
            "email": "johndoe@email.com",
            "password": "password"
        }))
        .to_request()
}

#[actix_rt::test]
async fn test_create_account() {
    let app = get_app().await;
    let app = test::init_service(app).await;
    let req = create_john_doe();
    let resp = test::call_service(&app, req).await;
    // assert that the response is a 201
    assert_eq!(resp.status(), 201);
    // get the response body
    let body = test::read_body(resp).await;
    // convert the body to json
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let id = json["id"].as_str().unwrap();
    let token = json["token"].as_str().unwrap();

    // assert that the token is a valid jwt
    let jwt = decode_jwt(token.to_string(), "secret").unwrap();
    assert_eq!(jwt.sub, id);
    assert_eq!(jwt.email, "johndoe@email.com");
}

#[actix_rt::test]
async fn test_create_account_duplicate() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    // create two duplicate accounts
    let req = create_john_doe();
    let req_dup = create_john_doe();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let resp = test::call_service(&server, req_dup).await;
    assert_eq!(resp.status(), 409);
}

#[actix_rt::test]
async fn test_get_account_by_id() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_john_doe();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let id = json["id"].as_str().unwrap();
    let token = json["token"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/{}", id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&server, req).await;

    assert_eq!(resp.status(), 200);
    let body = test::read_body(resp).await;

    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    assert_eq!(json["name"], "John Doe");
    assert_eq!(json["email"], "johndoe@email.com");
}

#[actix_rt::test]
async fn test_get_account_unauthorized() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_john_doe();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let id = json["id"].as_str().unwrap();
    let token = json["token"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/{}", id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}wrong", token)))
        .to_request();
    let resp = test::call_service(&server, req).await;

    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_update_account() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_john_doe();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let id = json["id"].as_str().unwrap();
    let token = json["token"].as_str().unwrap();

    let req = test::TestRequest::patch()
        .uri(&format!("/{}", id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "path": "name",
            "value": "Jane Doe"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    assert_eq!(json["name"], "Jane Doe");
    assert_eq!(json["email"], "johndoe@email.com");
}

#[actix_rt::test]
async fn test_delete_account() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_john_doe();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let id = json["id"].as_str().unwrap();
    let token = json["token"].as_str().unwrap();

    let req = test::TestRequest::delete()
        .uri(&format!("/{}", id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&server, req).await;

    assert_eq!(resp.status(), 204);
}

#[actix_rt::test]
async fn test_account_login() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_john_doe();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({
            "email": "johndoe@email.com",
            "password": "password"
        }))
        .to_request();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let token = json["token"].as_str().unwrap();
    let jwt = decode_jwt(token.to_string(), "secret").unwrap();
    assert_eq!(jwt.email, "johndoe@email.com");
}

#[actix_rt::test]
async fn test_account_login_invalid_credentials() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_john_doe();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({
            "email": "johndoe@email.com",
            "password": "badpassword"
        }))
        .to_request();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 401);

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({
            "email": "wrong@email.com",
            "password": "password"
        }))
        .to_request();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 401);
}
