use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Query},
    HttpResponse,
};
use std::env;

use crate::auth::user_auth::AuthorizationService;
use crate::handlers::types::{
    AccountPatch, AuthResponse, Credentials, ErrorResponse, ExternalAccountQuery, VerificationCodeQuery, VerificationQuery,
};
use crate::utils;
use crate::{
    auth::jwt::{decode_google_token_id, encode_jwt, GoogleAuthClaims},
    repository::redis::RedisRepository,
};
use crate::{models::profile::Profile, models::user::User, repository::database::DatabaseRepository};

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
pub async fn get_account_by_id(db: Data<DatabaseRepository>, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    if id.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            message: "Invalid id".to_string(),
        });
    }

    match db.get_account(&id).await {
        Ok(acc) => {
            log::debug!("Account: {:?}", acc);
            HttpResponse::Ok().json(acc)
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
pub async fn update_account(db: Data<DatabaseRepository>, mut req: Json<AccountPatch>, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    if id.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            message: "Invalid id".to_string(),
        });
    }

    if req.path != "name" && req.path != "email" && req.path != "password" {
        return HttpResponse::BadRequest().body("Invalid path");
    }

    if req.path == "password" {
        req.value = utils::validation::generate_hash(&req.value);
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
                HttpResponse::NotFound().json(ErrorResponse {
                    message: "Account not found".to_string(),
                })
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[delete("")]
pub async fn delete_account(db: Data<DatabaseRepository>, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    if id.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            message: "Invalid id".to_string(),
        });
    }

    let update_result = db.delete_account(&id).await;

    match update_result {
        Ok(acc) => {
            if acc.deleted_count == 1 {
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::NotFound().json(ErrorResponse {
                    message: "Account not found".to_string(),
                })
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
pub async fn create_account(db: Data<DatabaseRepository>, redis: Data<RedisRepository>, acc: Json<User>) -> HttpResponse {
    let user = db.get_account_by_email(&acc.email).await;
    match user {
        Ok(user) => {
            if user.is_some() {
                return HttpResponse::Conflict().json(ErrorResponse {
                    message: "Account already exists".to_string(),
                });
            }
        }
        Err(e) => {
            log::error!("Error: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Internal Server Error".to_string(),
            });
        }
    }

    // Check if account was verified by looking up the verification code
    // status in the redis cache
    let val = redis.get(&acc.email).await.unwrap();
    if val.is_empty() {
        log::debug!("Account has not been submitted for verification or verification window has expired. Please try to verify again.");
        return HttpResponse::BadRequest()
            .body("Account has not been submitted for verification or verification window has expired. Please try to verify again.");
    }
    if val.split(":").collect::<Vec<&str>>()[1] == "pending" {
        log::debug!("Account has not been verified yet");
        return HttpResponse::BadRequest().body("Account has not been verified yet");
    }

    // TODO: Move this to a middleware
    // Checks if the password is valid since it is optional in the User model
    // and we don't want to store an empty password
    // It is optional because we can create an account with an external provider
    let password = match acc.password.to_owned() {
        Some(p) => p,
        None => {
            log::debug!("Password is required");
            return HttpResponse::BadRequest().json(ErrorResponse {
                message: "Password is required".to_string(),
            });
        }
    };
    match utils::validation::validate_signup(&acc.email, &password) {
        Ok(_) => (),
        Err(e) => {
            log::debug!("Invalid signup: {}", e);
            return HttpResponse::BadRequest().json(ErrorResponse { message: e.to_string() });
        }
    };

    let secret = env::var("JWT_SECRET").unwrap(); // set this to global variable
    let hash_password = utils::validation::generate_hash(&password);

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

    let id = result.as_ref().unwrap().inserted_id.as_object_id().unwrap().to_hex();
    let domain = env::var("DOMAIN").unwrap();
    let app_name = env::var("APP_NAME").unwrap();
    let token = encode_jwt(app_name, id.to_owned(), domain, &secret);

    let response = AuthResponse { id, token };

    // Delete the verification code from the redis cache
    redis.del(&acc.email).await.unwrap();

    // Return early if we are in test environment
    if std::env::var("ENV").unwrap() == "test" {
        return match result {
            Ok(_result) => HttpResponse::Created().json(response),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        };
    }

    match utils::sendgrid::send_account_created(acc.email.as_str(), acc.name.as_str()).await {
        Ok(_) => (),
        Err(e) => log::error!("Error sending email: {:?}", e),
    }

    match result {
        Ok(_result) => HttpResponse::Created().json(response),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { message: e.to_string() }),
    }
}

#[post("/auth/login")]
pub async fn login_account(db: Data<DatabaseRepository>, cred: Json<Credentials>) -> HttpResponse {
    let user = db.get_account_by_email(&cred.email).await;
    let account = match user {
        Ok(user) => match user {
            Some(user) => user,
            None => {
                return HttpResponse::NotFound().json(ErrorResponse {
                    message: "Account not found".to_string(),
                })
            }
        },
        Err(e) => {
            log::error!("Error: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Internal Server Error".to_string(),
            });
        }
    };

    // if password is None, then the account was created with an external provider
    // and we can't login with it
    if account.password.is_none() {
        return HttpResponse::Unauthorized().body("Invalid password");
    }

    if utils::validation::verify_hash(&cred.password, account.password.as_ref().unwrap()) == false {
        return HttpResponse::Unauthorized().body("Invalid password");
    }

    let id = account.id.unwrap().to_hex();
    let secret = env::var("JWT_SECRET").unwrap();
    let domain = env::var("DOMAIN").unwrap();
    let app_name = env::var("APP_NAME").unwrap();
    let token = encode_jwt(app_name, id.to_owned(), domain, &secret);

    HttpResponse::Ok().json(AuthResponse { id, token })
}

#[post("/auth/google")]
pub async fn authenticate_external_account(db: Data<DatabaseRepository>, query: Query<ExternalAccountQuery>) -> HttpResponse {
    let secret = env::var("JWT_SECRET").unwrap();
    let domain = env::var("DOMAIN").unwrap();
    let app_name = env::var("APP_NAME").unwrap();

    let token = query.token_id.to_owned();
    let google_claims: GoogleAuthClaims = match decode_google_token_id(&token).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::BadRequest().json(e.to_string()),
    };
    let email = google_claims.email;

    // Check if the account already exists
    match db.get_account_by_email(&email).await {
        Ok(user) => {
            match user {
                Some(user) => {
                    // Account exists, returning token
                    let id = user.id.unwrap().to_hex();
                    let token = encode_jwt(app_name, id.to_owned(), domain, &secret);
                    HttpResponse::Ok().json(AuthResponse { id, token })
                }
                None => {
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

                    let id = result.as_ref().unwrap().inserted_id.as_object_id().unwrap().to_hex();
                    let token = encode_jwt(app_name, id.to_owned(), domain, &secret);

                    let response = AuthResponse { id, token };

                    match result {
                        Ok(_result) => HttpResponse::Created().json(response),
                        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { message: e.to_string() }),
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Error: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Internal Server Error".to_string(),
            });
        }
    }
}

/// Route to generate a verification code and send it to the user's email
/// The route generates a 6 digit code, stores it in a Redis cache
/// and sends it to the user's email
///
/// The code is valid for 10 minutes and is stored in the following
/// key-value format:
/// ```
/// email -> code:status
/// ```
///
/// The status is either `pending` or `used`
#[post("/auth/verification-code")]
pub async fn get_verification_code(db: Data<DatabaseRepository>, redis: Data<RedisRepository>, query: Query<VerificationCodeQuery>) -> HttpResponse {
    let name = query.name.to_owned();
    let email = query.email.to_owned();

    // Check if email exists in the database, if it does, return an error
    let user = db.get_account_by_email(&email).await;
    match user {
        Ok(user) => {
            if user.is_some() {
                return HttpResponse::Conflict().json(ErrorResponse {
                    message: "Email already exists".to_string(),
                });
            }
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Internal Server Error".to_string(),
            });
        }
    }

    let code = utils::validation::generate_verification_code();
    let status = "pending";
    let expiration_time = utils::validation::get_expiration_time(10);
    let value = format!("{}:{}", code, status);

    let result = redis.set(&email, &value).await;
    match result {
        Ok(_) => (),
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    }
    redis.expire(&email, expiration_time).await.unwrap();

    // Return early if in test environment
    if env::var("ENV").unwrap() == "test" {
        return HttpResponse::Ok().json(ErrorResponse {
            message: "Verification code sent".to_string(),
        });
    }

    let result = utils::sendgrid::send_email_verification(&email, &name, &code).await;
    match result {
        Ok(_) => HttpResponse::Ok().json(ErrorResponse {
            message: "Verification code sent".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { message: e.to_string() }),
    }
}

/// Route to verify a user's email given a verification code
/// The route checks if the code is valid and if it has not been used
/// If the code is valid and has not been used, it sets the token to `used`
/// and returns a `204` no content response.
/// If the code is invalid or has been used, it returns a `400`
/// bad request response
#[post("/auth/verify-email")]
pub async fn verify_email(redis: Data<RedisRepository>, query: Query<VerificationQuery>) -> HttpResponse {
    let email = query.email.to_owned();
    let code = query.code.to_owned();

    let result = redis.get(&email).await;
    match result {
        Ok(value) => {
            let parts: Vec<&str> = value.split(':').collect();
            let stored_code = parts[0];
            let status = parts[1];

            if stored_code == code && status == "pending" {
                let new_value = format!("{}:{}", stored_code, "used");
                redis.set(&email, &new_value).await.unwrap();
                HttpResponse::Ok().json("Email verified")
            } else if status == "used" {
                HttpResponse::BadRequest().json(ErrorResponse {
                    message: "This code has already been used".to_string(),
                })
            } else {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    message: "Invalid code. Please use a valid code.".to_string(),
                })
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { message: e.to_string() }),
    }
}
