use crate::{
    repository::account_repository::AccountRepository, 
    models::Account};
use actix_web::{
    web::{Data, Json, Path},
    get, post, delete, put,
    HttpResponse};
use log;

#[post("")]
pub async fn create_account(db: Data<AccountRepository>, acc: Json<Account>) -> HttpResponse {
    log::info!("Creating account: {:?}", acc);
    let data = Account {
        id: None,
        name: acc.name.to_owned(),
        email: acc.email.to_owned(),
        password: acc.password.to_owned(),
    };
    let acc = db.create_account(data).await;
    match acc {
        Ok(acc) => HttpResponse::Ok().json(acc),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/{id}")]
pub async fn get_account_by_id(db: Data<AccountRepository>, path: Path<String>) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }
    log::info!("Getting account by id: {:?}", id);
    let acc = db.get_account(&id).await;
    log::info!("Account: {:?}", acc);
    match acc {
        Ok(acc) => HttpResponse::Ok().json(acc),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}


#[put("/{id}")]
pub async fn update_account(db: Data<AccountRepository>, acc: Json<Account>) -> HttpResponse {
    HttpResponse::Ok().body("Hello there!")
}

#[delete("/{id}")]
pub async fn delete_account(db: Data<AccountRepository>, acc: Json<Account>) -> HttpResponse {
    HttpResponse::Ok().body("Hello there!")
}
