use crate::{repository::account_repository::AccountRepository, models::Account};
use actix_web::{
    web::{Data, Json, Path},
    get, post, delete, put,
    HttpResponse};
use mongodb::bson::oid::ObjectId;
use log;

#[post("")]
pub async fn create_account(db: Data<AccountRepository>, acc: Json<Account>) -> HttpResponse {
    // check if account already exists
    let exists = db.get_account_by_email(&acc.email).await;
    if exists.is_ok() {
        return HttpResponse::BadRequest().body("Account already exists");
    }
    
    let data = Account {
        id: None,
        name: acc.name.to_owned(),
        email: acc.email.to_owned(),
        password: acc.password.to_owned(),
        date_created: Some(chrono::Utc::now().timestamp()),
        date_updated: Some(chrono::Utc::now().timestamp()),
    };

    let acc = db.create_account(data).await;
    log::info!("Created account: {:?}", acc);
    match acc {
        Ok(acc) => HttpResponse::Ok().json(acc),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/{id}")]
pub async fn get_account_by_id(db: Data<AccountRepository>, path: Path<String>) -> HttpResponse {
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
pub async fn update_account(db: Data<AccountRepository>, path: Path<String>, acc: Json<Account>) -> HttpResponse {
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
pub async fn delete_account(db: Data<AccountRepository>, path: Path<String>) -> HttpResponse {
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