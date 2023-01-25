use crate::{repository::account_repository::AccountRepository, models::account::{Account, AccountResponse, Credentials}};
use crate::auth::jwt::encode_jwt;
use crate::auth::user_auth::AuthorizationService;
use crate::auth::utils;
use std::env;
use actix_web::{web::{Data, Json, Path}, get, post, delete, put, HttpResponse};
use mongodb::bson::oid::ObjectId;
use log;

#[post("/create")]
pub async fn create_account(db: Data<AccountRepository>, acc: Json<Account>) -> HttpResponse {
    let exists = db.get_account_by_email(&acc.email).await;
    if exists.is_ok() {
        return HttpResponse::BadRequest().body("Account already exists");
    }

    let hash_password = utils::generate_hash(&acc.password);

    let data = Account {
        id: None,
        name: acc.name.to_owned(),
        email: acc.email.to_owned(),
        password: hash_password.to_owned(),
        date_created: Some(chrono::Utc::now().timestamp()),
        date_updated: Some(chrono::Utc::now().timestamp()),
    };

    let result = db.create_account(data).await;
    log::info!("Created account: {:?}", result);

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let id = result.as_ref().unwrap().inserted_id.as_object_id().unwrap().to_hex();
    let token = encode_jwt(id.to_owned(), acc.email.to_owned(), &secret);

    let response = AccountResponse {
        id,
        token,
    };

    match result {
        Ok(_result) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/{id}")]
pub async fn get_account_by_id(db: Data<AccountRepository>, path: Path<String>, _auth: AuthorizationService) -> HttpResponse {
    // TODO: Don't send password back to client
    log::info!("Getting account by id: {:?}", path);
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let acc = db.get_account(&id).await;

    match acc {
        Ok(acc) => HttpResponse::Ok().json(acc),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}


#[put("/{id}")]
pub async fn update_account(db: Data<AccountRepository>, path: Path<String>, acc: Json<Account>, _auth: AuthorizationService) -> HttpResponse {
    log::info!("Updating account by id: {:?}", path);
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let data = Account {
        id: Some(ObjectId::parse_str(&id).unwrap()),
        name: acc.name.to_owned(),
        email: acc.email.to_owned(),
        password: acc.password.to_owned(),
        date_created: acc.date_created.to_owned(),
        date_updated: Some(chrono::Utc::now().timestamp()),
    };

    let update_result = db.update_account(&id, data).await;

    match update_result {
        Ok(acc) => {
            if acc.matched_count == 1 {
                let updated_user_info = db.get_account(&id).await;
                match updated_user_info {
                    Ok(acc) => HttpResponse::Ok().json(acc),
                    Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
                }
            } else {
                HttpResponse::BadRequest().body("Account not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[delete("/{id}")]
pub async fn delete_account(db: Data<AccountRepository>, path: Path<String>, _auth: AuthorizationService) -> HttpResponse {
    log::info!("Deleting account by id: {:?}", path);

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
pub async fn login_account(db: Data<AccountRepository>, cred: Json<Credentials>) -> HttpResponse {
    log::info!("Logging in account: {:?}", cred);

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

    let response = AccountResponse {
        id,
        token,
    };

    HttpResponse::Ok().json(response)
}
