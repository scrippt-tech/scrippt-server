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
use server::repository::{database::DatabaseRepository, redis::RedisRepository};
// use std::env;
use chrono;
use more_asserts::*;
use server::auth::jwt::{decode_google_token_id, decode_jwt};
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
    let db = DatabaseRepository::new("mongodb://localhost:27017").await;
    let redis = RedisRepository::new("redis://localhost:6379");
    let _ = db.drop_database().await;
    App::new()
        .wrap(middleware::NormalizePath::trim())
        .wrap(middleware::Logger::default())
        .app_data(web::Data::new(db))
        .app_data(web::Data::new(redis))
        .service(
            web::scope("/account")
                .service(create_account)
                .service(authenticate_external_account)
                .service(get_account_by_id)
                .service(update_account)
                .service(delete_account)
                .service(login_account)
                .service(get_verification_code)
                .service(verify_email),
        )
}

async fn create_some_account(name: &str, email: &str) -> actix_http::Request {
    let redis = RedisRepository::new("redis://localhost:6379");
    redis.set(email, "123456:used").await.unwrap();
    test::TestRequest::post()
        .uri("/account/create/")
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
    let req = create_some_account("John Doe", "johndoe@email.com").await;
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

#[actix_rt::test]
#[ignore = "This test requires a valid google token id"]
async fn test_external_account() {
    let app = get_app().await;
    let app = test::init_service(app).await;
    std::env::set_var("GOOGLE_JWK_PATH", ".jwk");

    let google_token_id = "<token_goes_here>";
    let req = test::TestRequest::post()
        .uri(format!("/account/auth/google?token_id={}", google_token_id).as_str())
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();

    let id = json["id"].as_str().unwrap();
    let token = json["token"].as_str().unwrap();

    let google_jwt_claims = decode_google_token_id(google_token_id)
        .await
        .expect("failed to decode google token id");

    // get account and compare
    let req = test::TestRequest::get()
        .uri("/account/")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();

    log::debug!("{:?}", json["id"].as_str());

    assert_eq!(json["id"].as_str().unwrap(), id);
    assert_eq!(json["email"].as_str().unwrap(), google_jwt_claims.email);
    assert_eq!(json["name"].as_str().unwrap(), google_jwt_claims.name);

    // test login
    let req = test::TestRequest::post()
        .uri(format!("/account/auth/google?token_id={}", google_token_id).as_str())
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    // try to login with empty password
    let req = test::TestRequest::post()
        .uri("/account/auth/login")
        .set_json(serde_json::json!({
            "email": google_jwt_claims.email,
            "password": "".to_string(),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 401);

    // try to login with no password
    let req = test::TestRequest::post()
        .uri("/account/auth/login")
        .set_json(serde_json::json!({
            "email": google_jwt_claims.email,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 400);

    // assert that the token is a valid jwt
}

/// This test creates an account, then tries to create another account with the same email
///
/// It should fail with a 409 Conflict
#[actix_rt::test]
async fn test_create_account_duplicate() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    // create two duplicate accounts
    let req = create_some_account("John Doe", "johndoe@email.com").await;
    let req_dup = create_some_account("John Doe", "johndoe@email.com").await;

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let resp = test::call_service(&server, req_dup).await;
    assert_eq!(resp.status(), 409);
}

/// This test creates an account, then tries to create another account with the same email
/// but with different casing
///
/// It should fail with a 409 Conflict
#[actix_rt::test]
async fn test_create_account_duplicate_case_insensitive() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    // create two duplicate accounts
    let req = create_some_account("John Doe", "johndoe@email.com").await;
    let req_dup = create_some_account("John Doe", "Johndoe@email.com").await;

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
        .uri("/account/create/")
        .set_json(serde_json::json!({
            "name": "John Doe",
            "email": "johndoe@gmail.com"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 400);

    // create an account with an invalid email
    let req = test::TestRequest::post()
        .uri("/account/create/")
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
        .uri("/account/create/")
        .set_json(serde_json::json!({
            "name": "John Doe",
            "email": "johnn@email.com",
            "password": "bad"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 400);

    // Create account with no name
    let req = test::TestRequest::post()
        .uri("/account/create/")
        .set_json(serde_json::json!({
            "email": "jane@email.com",
            "password": "password"
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
    let req = create_some_account("John Doe", "johndoe@email.com").await;
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let id = json["id"].as_str().unwrap();
    let token = json["token"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri("/account/")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&server, req).await;

    assert_eq!(resp.status(), 200);
    let body = test::read_body(resp).await;

    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    assert_eq!(json["id"], id);
    assert_eq!(json["name"], "John Doe");
    assert_eq!(json["email"], "johndoe@email.com");
}

/// This test creates an account, then tries to get the account by id with invalid token credentials.
/// It tests sending no token, an invalid token, a wrong token type, and no token type
///
/// It should return a 401 Unauthorized for each
#[actix_rt::test]
async fn test_get_account_unauthorized() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe", "johndoe@email.com").await;
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let token = json["token"].as_str().unwrap();

    // No token
    let req = test::TestRequest::get().uri("/account/").to_request();
    let resp = test::call_service(&server, req).await;

    assert_eq!(resp.status(), 401);

    // Invalid token
    let req = test::TestRequest::get()
        .uri("/account/")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}wrong", token)))
        .to_request();
    let resp = test::call_service(&server, req).await;

    assert_eq!(resp.status(), 401);

    // Incorrect token type
    let req = test::TestRequest::get()
        .uri("/account/")
        .insert_header((header::AUTHORIZATION, format!("Basic {}", token)))
        .to_request();
    let resp = test::call_service(&server, req).await;

    assert_eq!(resp.status(), 401);

    // No token type
    let req = test::TestRequest::get()
        .uri("/account/")
        .insert_header((header::AUTHORIZATION, format!("{}", token)))
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
    let req = create_some_account("John Doe", "johndoe@email.com").await;
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let token = json["token"].as_str().unwrap();

    // Update the name
    let req = test::TestRequest::patch()
        .uri("/account/")
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
        .uri("/account/")
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
        .uri("/account/")
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
        .uri("/account/auth/login/")
        .set_json(serde_json::json!({
            "email": "janedoe@email.com",
            "password": "password"
        }))
        .to_request();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 401);

    // Login with the new password
    let req = test::TestRequest::post()
        .uri("/account/auth/login/")
        .set_json(serde_json::json!({
            "email": "janedoe@email.com",
            "password": "new-password"
        }))
        .to_request();

    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 200);
}

/// This test creates an account, then tries to delete the account
///
/// It verifies that the response is 204 No Content
#[actix_rt::test]
async fn test_delete_account() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe", "johndoe@email.com").await;
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let token = json["token"].as_str().unwrap();

    let req = test::TestRequest::delete()
        .uri("/account/")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&server, req).await;

    assert_eq!(resp.status(), 204);
}

/// This test creates an account, then tries to login with the account
///
/// It verifies that the response is 200 OK
/// It verifies that the response body contains the account id and token
/// It verifies that the token is valid
#[actix_rt::test]
async fn test_account_login() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe", "johndoe@email.com").await;
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::post()
        .uri("/account/auth/login/")
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
    let jwt = decode_jwt(
        token.to_string(),
        &std::env::var("JWT_SECRET").expect("JWT secret must be set"),
    )
    .unwrap();
    assert_eq!(jwt.iss, std::env::var("APP_NAME").unwrap());
    assert_eq!(jwt.sub, id);
    assert_eq!(jwt.aud, std::env::var("DOMAIN").unwrap());
    assert_le!(jwt.iat, chrono::Utc::now().timestamp() as usize);
    assert_ge!(jwt.exp, chrono::Utc::now().timestamp() as usize);
    assert_le!(jwt.nbf, chrono::Utc::now().timestamp() as usize);
    assert_eq!(jwt.jti.len(), 36);
}

/// This test creates an account, then tries to login with the account
/// using invalid credentials
///
/// It verifies that the response is 401 Unauthorized
#[actix_rt::test]
async fn test_account_login_invalid_credentials() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = create_some_account("John Doe", "johndoe@email.com").await;
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::post()
        .uri("/account/auth/login/")
        .set_json(serde_json::json!({
            "email": "johndoe@email.com",
            "password": "badpassword"
        }))
        .to_request();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 401);

    let req = test::TestRequest::post()
        .uri("/account/auth/login/")
        .set_json(serde_json::json!({
            "email": "wrong@email.com",
            "password": "password"
        }))
        .to_request();
    let resp = test::call_service(&server, req).await;
    assert_eq!(resp.status(), 404);
}

/// This tests the verification code endpoint
/// It verifies that the code was added to the redis cache
#[actix_rt::test]
async fn test_verification_code() {
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = test::TestRequest::post()
        .uri("/account/auth/verification-code?name=test&email=some@email.com")
        .to_request();
    let res = test::call_service(&server, req).await;
    assert_eq!(res.status(), 200);

    // Verify that the redis cache contains the email as a key
    let redis = RedisRepository::new("redis://localhost:6379");
    let value = redis.get("some@email.com").await.unwrap();
    let val = value.split(':').collect::<Vec<&str>>();
    let exists = redis.exists("some@email.com").await.unwrap();
    assert_eq!(val[0].len(), 6);
    assert_eq!(val[1], "pending");
    assert!(exists);
}

/// Create account integration test
/// Sends a verification code to the email address
/// Creates an account with the verification code
#[actix_rt::test]
async fn test_create_account_verified() {
    let email = "johndoe@gmail.com";
    let name = "John";
    let app = get_app().await;
    let server = test::init_service(app).await;
    let req = test::TestRequest::post()
        .uri(&format!(
            "/account/auth/verification-code?name={}&email={}",
            name, email
        ))
        .to_request();
    let res = test::call_service(&server, req).await;
    assert_eq!(res.status(), 200);

    // Verify that the redis cache contains the email as a key
    let redis = RedisRepository::new("redis://localhost:6379");
    let value = redis.get(email).await.unwrap();
    let val = value.split(':').collect::<Vec<&str>>();
    let exists = redis.exists(email).await.unwrap();
    assert_eq!(val[0].len(), 6);
    assert_eq!(val[1], "pending");
    assert!(exists);

    // Try to create an account, but fail because the email is not verified
    let req = test::TestRequest::post()
        .uri("/account/create/")
        .set_json(serde_json::json!({
            "name": name,
            "email": email,
            "password": "password",
        }))
        .to_request();
    let res = test::call_service(&server, req).await;
    assert_eq!(res.status(), 400);
    // Get the response body and log the message
    let body = test::read_body(res).await;
    log::debug!("body: {}", std::str::from_utf8(&body).unwrap());

    // submit code for verification
    let req = test::TestRequest::post()
        .uri(&format!(
            "/account/auth/verify-email?email={}&code={}",
            email, val[0]
        ))
        .to_request();
    let res = test::call_service(&server, req).await;
    assert_eq!(res.status(), 200);

    // Create account
    let req = test::TestRequest::post()
        .uri("/account/create/")
        .set_json(serde_json::json!({
            "name": name,
            "email": email,
            "password": "password",
        }))
        .to_request();
    let res = test::call_service(&server, req).await;

    assert_eq!(res.status(), 201);

    // Verify that the redis cache no longer contains the email as a key
    let exists = redis.exists(email).await.unwrap();
    assert!(!exists);
}

/// This test sends a verification code to an email address
/// Then tries to verify the email address with an invalid code
#[actix_rt::test]
async fn test_invalid_verification() {
    let email = ""; // TODO: Fill in an actual email address for testing
    let name = ""; // TODO: Fill in an actual name for testing
    let app = get_app().await;
    let server = test::init_service(app).await;

    // Try to create an unverified account
    let req = test::TestRequest::post()
        .uri("/account/create/")
        .set_json(serde_json::json!({
            "name": name,
            "email": email,
            "password": "password",
        }))
        .to_request();
    let res = test::call_service(&server, req).await;
    assert_eq!(res.status(), 400);
    let body = test::read_body(res).await;
    log::debug!(
        "Unverified account body: {}",
        std::str::from_utf8(&body).unwrap()
    );

    let req = test::TestRequest::post()
        .uri(&format!(
            "/account/auth/verify-email?email={}&name={}",
            email, "123456"
        ))
        .to_request();
    let res = test::call_service(&server, req).await;
    assert_eq!(res.status(), 400);

    let body = test::read_body(res).await;
    log::debug!(
        "Invalid verification code body: {}",
        std::str::from_utf8(&body).unwrap()
    );

    let req = test::TestRequest::post()
        .uri(&format!(
            "/account/auth/verification-code?email={}&name={}",
            email, name
        ))
        .to_request();
    let res = test::call_service(&server, req).await;
    assert_eq!(res.status(), 200);

    // Verify that the redis cache contains the email as a key
    let redis = RedisRepository::new("redis://localhost:6379");
    let value = redis.get(email).await.unwrap();
    let val = value.split(':').collect::<Vec<&str>>();
    let exists = redis.exists(email).await.unwrap();
    assert_eq!(val[0].len(), 6);
    assert_eq!(val[1], "pending");
    assert!(exists);

    // submit code for verification
    let req = test::TestRequest::post()
        .uri(&format!(
            "/account/auth/verify-email?email={}&code={}",
            email, "123456"
        ))
        .to_request();
    let res = test::call_service(&server, req).await;
    assert_eq!(res.status(), 401);

    let body = test::read_body(res).await;
    log::debug!(
        "Invalid verification code body: {}",
        std::str::from_utf8(&body).unwrap()
    );
}
