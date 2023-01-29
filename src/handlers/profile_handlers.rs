use actix_web::{web::{Data, Json, Path}, put, HttpResponse};
use crate::models::profile::{ProfileInfo};

use crate::repository::db::DatabaseRepository;
use crate::auth::user_auth::AuthorizationService;

#[put("/{id}")]
pub async fn update_profile(db: Data<DatabaseRepository>, path: Path<String>, profile: Json<ProfileInfo>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let profile = ProfileInfo {
        education: profile.education.to_owned(),
        experience: profile.experience.to_owned(),
        skills: profile.skills.to_owned(),
    };

    let date = chrono::Utc::now().timestamp();

    let update_result = db.update_profile(&id, &profile, date).await;

    match update_result {
        Ok(profile) => {
            if profile.matched_count == 1 {
                HttpResponse::Ok().json(profile)
            } else {
                HttpResponse::BadRequest().body("Profile not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}