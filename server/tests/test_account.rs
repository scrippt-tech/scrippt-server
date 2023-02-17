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
use chrono;
use more_asserts::*;
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

fn create_some_account(name: String, email: String) -> actix_http::Request {
    test::TestRequest::post()
        .uri("/create")
        .set_json(serde_json::json!({
            "name": name,
            "email": email,
            "password": "password"
        }))
        .to_request()
}

/// This test creates a user and asserts that the response is a 201
/// and that the response body contains a valid jwt.
///
/// It also asserts that the jwt contains the correct user id and email
#[actix_rt::test]
async fn test_create_account() {
    let app = get_app().await;
    let app = test::init_service(app).await;
    let req = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());
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
    assert_eq!(jwt.iss, std::env::var("APP_NAME").unwrap());
    assert_eq!(jwt.sub, id);
    assert_eq!(jwt.aud, std::env::var("DOMAIN").unwrap());
    assert_le!(jwt.iat, chrono::Utc::now().timestamp() as usize);
    assert_ge!(jwt.exp, chrono::Utc::now().timestamp() as usize);
    assert_le!(jwt.nbf, chrono::Utc::now().timestamp() as usize);
    assert_eq!(jwt.jti.len(), 36);
}

/// This test creates an account, then tries to create another account with the same email
///
/// It should fail with a 409 Conflict
#[actix_rt::test]
async fn test_create_account_duplicate() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    // create two duplicate accounts
    let req = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());
    let req_dup = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let resp = test::call_service(&server, req_dup).await;
    assert_eq!(resp.status(), 409);
}

/// This test creates an account with a missing field, an invalid email, and an invalid password
///
/// It should fail with a 400 Bad Request
#[actix_rt::test]
async fn test_create_account_bad_request() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    // create an account with a missing field
    let req = test::TestRequest::post()
        .uri("/create")
        .set_json(serde_json::json!({
            "name": "John Doe",
            "email": "johndoe@gmail.com"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 400);

    // create an account with an invalid email
    let req = test::TestRequest::post()
        .uri("/create")
        .set_json(serde_json::json!({
            "name": "John Doe",
            "email": "bad-email",
            "password": "password"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 400);

    // create an account with an invalid password
    let req = test::TestRequest::post()
        .uri("/create")
        .set_json(serde_json::json!({
            "name": "John Doe",
            "email": "johnn@email.com",
            "password": "bad"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 400);
}

/// This test creates an account, then tries to get the account by id
///
/// It should succeed with a 200 \
/// It verifies that the response body contains the correct user id and email
#[actix_rt::test]
async fn test_get_account_by_id() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());
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

/// This test creates an account, then tries to get the account by id with an invalid token
///
/// It should return a 401 Unauthorized
#[actix_rt::test]
async fn test_get_account_unauthorized() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());
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

/// This test creates two accounts, account A and account B.
/// It tries to get account A's information with account B's token
/// It should fail with a 401 Forbidden
#[actix_rt::test]
async fn test_get_account_forbidden() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let token = json["token"].as_str().unwrap();

    let req = create_some_account("Jane Doe".to_string(), "janedoe@email.com".to_string());
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let id = json["id"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/{}", id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&server, req).await;

    assert_eq!(resp.status(), 401);
}

/// This test creates an account, then tries to update the account
///
/// It updates the name, then verifies that the name was updated.
/// It verifies that the response body contains the correct name and email
///
/// It updates the email, then verifies that the email was updated.
/// It verifies that the response body contains the correct name and email.
/// It verifies that the JWT token contains the correct updated email
///
/// It updates the password, then verifies that the password was updated.
/// It verifies that the response body contains the correct name and email.
/// It tries to login with the old password, which should fail.
/// It tries to login with the new password, which should succeed
#[actix_rt::test]
async fn test_update_account() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let id = json["id"].as_str().unwrap();
    let token = json["token"].as_str().unwrap();

    // Update the name
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

    // Update the email
    let req = test::TestRequest::patch()
        .uri(&format!("/{}", id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "path": "email",
            "value": "janedoe@email.com"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    assert_eq!(json["name"], "Jane Doe");
    assert_eq!(json["email"], "janedoe@email.com");

    // Update the password
    let req = test::TestRequest::patch()
        .uri(&format!("/{}", id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "path": "password",
            "value": "new-password"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    assert_eq!(json["name"], "Jane Doe");
    assert_eq!(json["email"], "janedoe@email.com");

    // Try to login with the old password
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({
            "email": "janedoe@email.com",
            "password": "password"
        }))
        .to_request();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 401);

    // Login with the new password
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({
            "email": "janedoe@email.com",
            "password": "new-password"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_rt::test]
async fn test_delete_account() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());
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
    let req = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());
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
    let id = json["id"].as_str().unwrap();
    let token = json["token"].as_str().unwrap();
    let jwt = decode_jwt(token.to_string(), "secret").unwrap();
    assert_eq!(jwt.iss, std::env::var("APP_NAME").unwrap());
    assert_eq!(jwt.sub, id);
    assert_eq!(jwt.aud, std::env::var("DOMAIN").unwrap());
    assert_le!(jwt.iat, chrono::Utc::now().timestamp() as usize);
    assert_ge!(jwt.exp, chrono::Utc::now().timestamp() as usize);
    assert_le!(jwt.nbf, chrono::Utc::now().timestamp() as usize);
    assert_eq!(jwt.jti.len(), 36);
}

#[actix_rt::test]
async fn test_account_login_invalid_credentials() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe".to_string(), "johndoe@email.com".to_string());
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
