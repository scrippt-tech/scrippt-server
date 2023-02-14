use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::env;

use crate::auth::jwt::encode_jwt;
use crate::auth::user_auth::AuthorizationService;
use crate::auth::utils;
use crate::{
    models::profile::Profile,
    models::user::{AccountPatch, User},
    repository::database::DatabaseRepository,
};

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    pub id: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub profile: Profile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

/// API route to create a user with an empty profile and no documents
///
/// ### Request body:
/// ```
/// {
///    "name": String,
///    "email": String,
///    "password": String
/// }
/// ```
///
/// ### Response body (if successful):
/// ```
/// 201 Created
/// {
///     "id": String,
///     "token": String
/// }
/// ```
#[post("/create")]
pub async fn create_account(db: Data<DatabaseRepository>, acc: Json<User>) -> HttpResponse {
    let exists = db.get_account_by_email(&acc.email).await;

    if exists.is_ok() {
        return HttpResponse::Conflict().body("Account already exists");
    }

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let hash_password = utils::generate_hash(&acc.password);

    let empty_profile = Profile {
        education: vec![],
        experience: vec![],
        skills: vec![],
        date_updated: Some(chrono::Utc::now().timestamp()),
    };

    let data = User {
        id: None,
        name: acc.name.to_owned(),
        email: acc.email.to_owned(),
        password: hash_password.to_owned(),
        profile: Some(empty_profile),
        documents: Some(vec![]),
        date_created: Some(chrono::Utc::now().timestamp()),
        date_updated: Some(chrono::Utc::now().timestamp()),
    };

    let result = db.create_account(data).await;

    let id = result
        .as_ref()
        .unwrap()
        .inserted_id
        .as_object_id()
        .unwrap()
        .to_hex();
    let token = encode_jwt(id.to_owned(), acc.email.to_owned(), &secret);

    let response = AuthResponse { id, token };

    match result {
        Ok(_result) => HttpResponse::Created().json(response),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

/// API route to get a user's account by id. Returns a user's account information.
///
/// ### Response body (if successful):
/// ```
/// 200 OK
/// {
///   "id": String,
///   "name": String,
///   "email": String,
///   "profile": Object
/// }
/// ```
#[get("/{id}")]
pub async fn get_account_by_id(
    db: Data<DatabaseRepository>,
    path: Path<String>,
    _auth: AuthorizationService,
) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let acc = db.get_account(&id).await;

    match acc {
        Ok(acc) => {
            let response = UserResponse {
                id: acc.id.unwrap().to_hex(),
                name: acc.name,
                email: acc.email,
                profile: acc.profile.unwrap_or_default(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

/// API route to update a user's account. Request takes in a one or multiple of the following fields:
///
/// - name
/// - email
/// - password
///
/// ### Request body:
/// ```
/// {
///   "name" | "email" | "password": String,
///   ...
/// }
/// ```
///
/// ### Response body (if successful):
/// ```
/// 200 OK
/// {
///    "id": String,
///    "name": String,
///    "email": String,
/// }
/// ```
#[patch("/{id}")]
pub async fn update_account(
    db: Data<DatabaseRepository>,
    path: Path<String>,
    mut req: Json<AccountPatch>,
    _auth: AuthorizationService,
) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    if req.path != "name" && req.path != "email" && req.path != "password" {
        return HttpResponse::BadRequest().body("Invalid path");
    }

    if req.path == "password" {
        req.value = utils::generate_hash(&req.value);
    }

    let to_update = AccountPatch {
        path: req.path.to_owned(),
        value: req.value.to_owned(),
    };

    let update_result = db.update_account(&id, to_update).await;

    let res = db.get_account(&id).await.unwrap();

    match update_result {
        Ok(acc) => {
            if acc.matched_count == 1 {
                HttpResponse::Ok().json(res)
            } else {
                HttpResponse::BadRequest().body("Account not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[delete("/{id}")]
pub async fn delete_account(
    db: Data<DatabaseRepository>,
    path: Path<String>,
    _auth: AuthorizationService,
) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let acc = db.delete_account(&id).await;

    match acc {
        Ok(acc) => HttpResponse::Ok().json(acc),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

// Authentication Handlers
#[post("/login")]
pub async fn login_account(db: Data<DatabaseRepository>, cred: Json<Credentials>) -> HttpResponse {
    let exists = db.get_account_by_email(&cred.email).await;
    if exists.is_err() {
        return HttpResponse::BadRequest().body("Account does not exist");
    }

    let account = exists.unwrap();
    if utils::verify_hash(&cred.password, &account.password) == false {
        return HttpResponse::BadRequest().body("Invalid password");
    }

    let id = account.id.unwrap().to_hex();
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode_jwt(id.to_owned(), cred.email.to_owned(), &secret);

    let response = AuthResponse { id, token };

    HttpResponse::Ok().json(response)
}

// Tests for the account handlers.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::database::DatabaseRepository;
    use actix_http::body::MessageBody;
    use actix_service::ServiceFactory;
    use actix_web::{
        dev::{ServiceRequest, ServiceResponse},
        error::Error,
        http, middleware, test, App,
    };

    fn get_app() -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = Error,
        >,
    > {
        let db = DatabaseRepository::new("mongodb://localhost:27017", "localhost".to_string());
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(db)
            .service(create_account)
            .service(get_account_by_id)
            .service(update_account)
            .service(delete_account)
            .service(login_account)
    }

    // #[actix_web::test]
    // async fn test_response() {
    //     let app = test::init_service(
    //         App::new().service(web::resource("/test").to(|| async { HttpResponse::Ok() })),
    //     )
    //     .await;

    //     // Create request object
    //     let req = test::TestRequest::with_uri("/test").to_request();

    //     // Call application
    //     let res = test::call_service(&app, req).await;
    //     assert_eq!(res.status(), http::StatusCode::OK);
    // }

    #[actix_rt::test]
    async fn test_create_account() {
        let app = test::init_service(get_app()).await;
        let req = test::TestRequest::post()
            .uri("/create")
            .set_json(serde_json::json!({
                "name": "John Doe",
                "email": "johndoe@email.com",
                "password": "password"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);
    }

    #[actix_rt::test]
    async fn test_create_account_duplicate() {
        let server = test::init_service(get_app()).await;
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
        assert_eq!(resp.status(), 409);
    }
}
