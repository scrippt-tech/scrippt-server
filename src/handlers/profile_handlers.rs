use actix_web::{web::{Data, Json, Path}, put, HttpResponse};
use serde::{Serialize, Deserialize};

use crate::models::profile::{ProfileInfo};

use crate::repository::db::DatabaseRepository;
use crate::auth::user_auth::AuthorizationService;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileUpdate {
    profile: ProfileInfo,
    op: String
}

async fn create_profile(db: Data<DatabaseRepository>, id: String, data: &ProfileInfo, date: i64) -> HttpResponse {
    let update_result = db.create_profile(&id, data.to_owned(), date).await;

    match update_result {
        Ok(data) => {
            if data.matched_count == 1 {
                HttpResponse::Ok().json(data)
            } else {
                HttpResponse::BadRequest().body("Profile not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn update_profile(db: Data<DatabaseRepository>, id: String, data: &ProfileInfo, date: i64) -> HttpResponse {
    let update_result = db.update_profile(&id, data.to_owned(), date).await;

    match update_result {
        Ok(data) => {
            if data.matched_count == 1 {
                HttpResponse::Ok().json(data)
            } else {
                HttpResponse::BadRequest().body("Profile not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[put("/{id}")]
pub async fn profile(db: Data<DatabaseRepository>, path: Path<String>, profile: Json<ProfileUpdate>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    if profile.op == "new" {
        let date = chrono::Utc::now().timestamp();
        return create_profile(db, id, &profile.profile, date).await;
    } else if profile.op == "update" {
        let date = chrono::Utc::now().timestamp();
        return update_profile(db, id, &profile.profile, date).await;
    } else {
        return HttpResponse::BadRequest().body("Invalid operation");
    }
}