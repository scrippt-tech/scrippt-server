use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Query},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::env;

use crate::auth::jwt::{decode_google_token_id, encode_jwt, GoogleAuthClaims};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalAccountQuery {
    pub token_id: String,
}

/// API route to get a user's account by id. Returns a user's account information.
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
#[get("")]
pub async fn get_account_by_id(
    db: Data<DatabaseRepository>,
    auth: AuthorizationService,
) -> HttpResponse {
    let id = auth.id;
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

/// API route to update a user's account.
/// ### Request body:
/// ```
/// {
///   "path": "name" | "email" | "password",
///   "value": String
/// }
/// ```
/// ### Response body (if successful):
/// ```
/// 200 OK
/// {
///    "id": String,
///    "name": String,
///    "email": String,
/// }
/// ```
#[patch("")]
pub async fn update_account(
    db: Data<DatabaseRepository>,
    mut req: Json<AccountPatch>,
    auth: AuthorizationService,
) -> HttpResponse {
    let id = auth.id;
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

#[delete("")]
pub async fn delete_account(
    db: Data<DatabaseRepository>,
    auth: AuthorizationService,
) -> HttpResponse {
    let id = auth.id;
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
    log::info!("Account exists: {:?}", exists);
    match exists {
        Ok(_) => return HttpResponse::Conflict().body("Account already exists"),
        Err(_) => (),
    }

    // TODO: Move this to a middleware
    // Checks if the password is valid since it is optional in the User model
    // and we don't want to store an empty password
    // It is optional because we can create an account with an external provider
    let password = match acc.password.as_ref() {
        Some(p) => p,
        None => return HttpResponse::BadRequest().body("Password is required"),
    };
    match utils::validate_signup(&acc.email, password) {
        Ok(_) => (),
        Err(e) => return HttpResponse::BadRequest().json(e.to_string()),
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set"); // set this to global variable
    let hash_password = utils::generate_hash(password);

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
        external_id: None,
        external_provider: None,
        password: Some(hash_password),
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
    let domain = env::var("DOMAIN").expect("DOMAIN must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let token = encode_jwt(app_name, id.to_owned(), domain, &secret);

    let response = AuthResponse { id, token };

    match result {
        Ok(_result) => HttpResponse::Created().json(response),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[post("/auth/login")]
pub async fn login_account(db: Data<DatabaseRepository>, cred: Json<Credentials>) -> HttpResponse {
    let exists = db.get_account_by_email(&cred.email).await;
    if exists.is_err() {
        return HttpResponse::Unauthorized().body("Account does not exist");
    }

    let account = exists.unwrap();

    // if password is None, then the account was created with an external provider
    // and we can't login with it
    if account.password.is_none() {
        return HttpResponse::Unauthorized().body("Invalid password");
    }

    if utils::verify_hash(&cred.password, account.password.as_ref().unwrap()) == false {
        return HttpResponse::Unauthorized().body("Invalid password");
    }

    let id = account.id.unwrap().to_hex();
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let domain = env::var("DOMAIN").expect("DOMAIN must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let token = encode_jwt(app_name, id.to_owned(), domain, &secret);

    HttpResponse::Ok().json(AuthResponse { id, token })
}

#[post("/auth/google")]
pub async fn authenticate_external_account(
    db: Data<DatabaseRepository>,
    query: Query<ExternalAccountQuery>,
) -> HttpResponse {
    let domain = env::var("DOMAIN").expect("DOMAIN must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let token = query.token_id.to_owned();
    let google_claims: GoogleAuthClaims = match decode_google_token_id(&token).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::BadRequest().json(e.to_string()),
    };
    let email = google_claims.email;
    log::info!("Email: {}", email);

    // Check if the account already exists
    let exists = db.get_account_by_email(&email).await;
    match exists {
        Ok(_) => {
            // Account exists, returning token
            let id = exists.unwrap().id.unwrap().to_hex();
            let token = encode_jwt(app_name, id.to_owned(), domain, &secret);
            HttpResponse::Ok().json(AuthResponse { id, token })
        }
        Err(_) => {
            // Account does not exist, creating new account
            let empty_profile = Profile {
                education: vec![],
                experience: vec![],
                skills: vec![],
                date_updated: Some(chrono::Utc::now().timestamp()),
            };

            let data = User {
                id: None,
                name: google_claims.name,
                email,
                external_id: Some(google_claims.sub),
                external_provider: Some("google".to_string()),
                password: None,
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
            let token = encode_jwt(app_name, id.to_owned(), domain, &secret);

            let response = AuthResponse { id, token };

            match result {
                Ok(_result) => HttpResponse::Created().json(response),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
    }
}
