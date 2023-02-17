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

    match utils::validate_signup(&acc.email, &acc.password) {
        Ok(_) => (),
        Err(e) => return HttpResponse::BadRequest().json(e.to_string()),
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set"); // set this to global variable
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
///   "path": "name" | "email" | "password",
///   "value": String
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

    let update_result = db.delete_account(&id).await;

    match update_result {
        Ok(acc) => {
            if acc.deleted_count == 1 {
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::BadRequest().body("Account not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

// Authentication Handlers
#[post("/login")]
pub async fn login_account(db: Data<DatabaseRepository>, cred: Json<Credentials>) -> HttpResponse {
    let exists = db.get_account_by_email(&cred.email).await;
    if exists.is_err() {
        return HttpResponse::Unauthorized().body("Account does not exist");
    }

    let account = exists.unwrap();
    if utils::verify_hash(&cred.password, &account.password) == false {
        return HttpResponse::Unauthorized().body("Invalid password");
    }

    let id = account.id.unwrap().to_hex();
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode_jwt(id.to_owned(), cred.email.to_owned(), &secret);

    let response = AuthResponse { id, token };

    HttpResponse::Ok().json(response)
}
