#![cfg(test)]

use actix_http::{body::MessageBody, header};
use actix_service::ServiceFactory;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::Error,
    middleware, test, web, App,
};
use assert_json_diff::assert_json_include;
use server::handlers::account_handlers::create_account;
use server::handlers::profile_handlers::change_profile;
use server::repository::{database::DatabaseRepository, redis::RedisRepository};
use std::sync::Once;

static INIT: Once = Once::new();

async fn get_app(
) -> App<impl ServiceFactory<ServiceRequest, Response = ServiceResponse<impl MessageBody>, Config = (), InitError = (), Error = Error>> {
    // set up the logger to debug
    INIT.call_once(env_logger::init);
    let db = DatabaseRepository::new("mongodb://localhost:27017").await;
    let redis = RedisRepository::new("redis://localhost:6379");
    let _ = db.drop_database().await;
    App::new()
        .wrap(middleware::NormalizePath::trim())
        .wrap(middleware::Logger::default())
        .app_data(web::Data::new(db))
        .app_data(web::Data::new(redis))
        .service(web::scope("/account").service(create_account))
        .service(web::scope("/profile").service(change_profile))
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

#[actix_rt::test]
async fn test_profile() {
    let app = get_app().await;
    let app = test::init_service(app).await;
    let req = create_some_account("John Doe", "johndoe@gmail.com").await;
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // get the jwt token from JSON response body
    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let token = json["token"].as_str().unwrap();

    let experience = serde_json::json!({
        "name": "Software Engineer",
        "type": "work",
        "at": "Google",
        "current": true,
        "description": "Worked on Google Cloud Platform",
    });

    let skill = serde_json::json!({
        "skill": "Rust",
    });

    let order = serde_json::json!([
        {
            "op": "add",
            "target": "experience",
            "value": {
                "type": "experience",
                "value": experience
            }
        },
        {
            "op": "add",
            "target": "skills",
            "value": {
                "type": "skills",
                "value": skill
            }
        }
    ]);

    // add profile
    let req = test::TestRequest::patch()
        .uri("/profile/")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&order)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    log::debug!("{:#?}", json);
    let added_experience = json["profile"]["experience"][0].as_object().unwrap();
    let added_skill = json["profile"]["skills"][0].as_object().unwrap();

    let experience_id = added_experience["field_id"].as_str().unwrap();
    // let skill_id = added_skill["field_id"].as_str().unwrap();

    assert_json_include!(actual: added_experience, expected: experience);
    assert_json_include!(actual: added_skill, expected: skill);

    // update profile
    let education = serde_json::json!({
        "school": "MIT",
        "degree": "Bachelors",
        "field_of_study": "Computer Science",
        "current": false,
        "description": "Graduated summa cum laude with a 4.0 GPA",
    });

    let order = serde_json::json!([
        {
            "op": "remove",
            "target": "experience",
            "value": {
                "type": "field_id",
                "value": experience_id
            }
        },
        {
            "op": "add",
            "target": "education",
            "value": {
                "type": "education",
                "value": education
            }
        }
    ]);

    let req = test::TestRequest::patch()
        .uri("/profile/")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&order)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();

    let added_education = json["profile"]["education"][0].as_object().unwrap();
    let removed_experience = json["profile"]["experience"].as_array().unwrap();

    assert_eq!(removed_experience.len(), 0);
    assert_json_include!(actual: added_education, expected: education);
}
