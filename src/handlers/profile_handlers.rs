use actix_web::{web::{Data, Json, Path}, get, post, delete, put, HttpResponse};

use crate::repository::db::DatabaseRepository;
use crate::models::profile::Profile;
use crate::auth::user_auth::AuthorizationService;

#[post("/{id}")]
pub async fn create_profile(db: Data<DatabaseRepository>, path: Path<String>, profile: Json<Profile>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    let exists = db.get_profile(&id).await;
    if exists.is_ok() {
        return HttpResponse::BadRequest().body("Profile already exists");
    }

    let data = Profile {
        account_id: Some(id),
        profile: profile.profile.to_owned(),
        date_created: Some(chrono::Utc::now().timestamp()),
        date_updated: Some(chrono::Utc::now().timestamp()),
    };

    let result = db.create_profile(data).await;
    log::info!("Created profile: {:?}", result);

    match result {
        Ok(_result) => HttpResponse::Ok().body("Created profile"),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/{id}")]
pub async fn get_profile_by_id(db: Data<DatabaseRepository>, path: Path<String>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let profile = db.get_profile(&id).await;

    match profile {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[put("/{id}")]
pub async fn update_profile(db: Data<DatabaseRepository>, path: Path<String>, profile: Json<Profile>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let data = Profile {
        account_id: Some(id.to_owned()),
        profile: profile.profile.to_owned(),
        date_created: profile.date_created.to_owned(),
        date_updated: Some(chrono::Utc::now().timestamp()),
    };

    let update_result = db.update_profile(&id, data).await;

    match update_result {
        Ok(profile) => {
            if profile.matched_count == 1 {
                let updated_profile = db.get_profile(&id).await;
                match updated_profile {
                    Ok(profile) => HttpResponse::Ok().json(profile),
                    Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
                }
            } else {
                HttpResponse::BadRequest().body("Profile not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[delete("/{id}")]
pub async fn delete_profile(db: Data<DatabaseRepository>, path: Path<String>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let delete_result = db.delete_profile(&id).await;

    match delete_result {
        Ok(profile) => {
            if profile.deleted_count == 1 {
                HttpResponse::Ok().body("Deleted profile")
            } else {
                HttpResponse::BadRequest().body("Profile not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}