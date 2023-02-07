use actix_web::{web::{Data, Json, Path}, get, post, delete, put, HttpResponse};
use serde::{Serialize, Deserialize};
use std::env;
use log;

use crate::{
    repository::database::DatabaseRepository, 
    models::user::{User, UserUpdate}, 
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

// TODO: Change this to a patch request
#[put("/{id}")]
pub async fn update_account(db: Data<DatabaseRepository>, path: Path<String>, acc: Json<User>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let data = UserUpdate {
        name: acc.name.to_owned(),
        email: acc.email.to_owned(),
        date_updated: chrono::Utc::now().timestamp(),
    };

    let update_result = db.update_account(&id, &data).await;

    match update_result {
        Ok(acc) => {
            if acc.matched_count == 1 {
                HttpResponse::Ok().json(data)
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
