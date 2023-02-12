use actix_web::{web::{Data, Json, Path}, get, post, delete, patch, HttpResponse};
use serde::{Serialize, Deserialize};
use std::env;

use crate::{
    repository::database::DatabaseRepository, 
    models::user::User, 
    models::profile::Profile,
};
use crate::auth::jwt::encode_jwt;
use crate::auth::user_auth::AuthorizationService;
use crate::auth::utils;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchRequest {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
    date_updated: Option<i64>,
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
/// 
/// ### Response body (if unsuccessful):
/// ```
/// 409 Conflict
/// "Account already exists"
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

    let id = result.as_ref().unwrap().inserted_id.as_object_id().unwrap().to_hex();
    let token = encode_jwt(id.to_owned(), acc.email.to_owned(), &secret);

    let response = AuthResponse {
        id,
        token,
    };

    match result {
        Ok(_result) => HttpResponse::Created().json(response),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/{id}")]
pub async fn get_account_by_id(db: Data<DatabaseRepository>, path: Path<String>, _auth: AuthorizationService) -> HttpResponse {
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
        },
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
/// 
/// ### Response body (if unsuccessful):
/// ```
/// 400 Bad Request
/// <error message>
/// ```
#[patch("/{id}")]
pub async fn update_account(db: Data<DatabaseRepository>, path: Path<String>, mut req: Json<PatchRequest>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let mut password_hash = String::new();
    if req.password.is_some() {
        password_hash = utils::generate_hash(&req.password.take().unwrap());
    }

    let update_data = User {
        id: None,
        name: req.name.take().unwrap(),
        email: req.email.take(),
        password: password_hash,
        profile: None,
        documents: None,
        date_created: None,
        date_updated: Some(chrono::Utc::now().timestamp()),
    };

    let update_result = db.update_account(&id, req).await;

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
pub async fn delete_account(db: Data<DatabaseRepository>, path: Path<String>, _auth: AuthorizationService) -> HttpResponse {

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

    let response = AuthResponse {
        id,
        token,
    };

    HttpResponse::Ok().json(response)
}
