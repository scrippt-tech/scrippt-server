use actix_web::{web::{Data, Json, Path}, patch, put, HttpResponse};
use serde::{Serialize, Deserialize};
use serde_json;

use crate::models::profile::Profile;
use crate::repository::database::DatabaseRepository;
use crate::auth::user_auth::AuthorizationService;

#[derive(Clone)]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProfilePatch {
    pub op: String,
    pub path: String,
    pub value: serde_json::Value,
}

/// # Create a new user profile
/// ## Request body:
/// ```
/// {
///    "education": Array,
///    "experience": Array,
///    "skills": Array,
/// }
/// ```
/// ## Response body:  (if successful)
/// ```
/// {
///   "education": Array,
///   "experience": Array,
///   "skills": Array,
///   "date_updated": Int,
/// }
/// ```
#[put("/{id}")]
pub async fn create_profile(db: Data<DatabaseRepository>, path: Path<String>, profile: Json<Profile>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let date = chrono::Utc::now().timestamp();

    let created = serde_json::json! ({
        "education": profile.education.to_owned(),
        "experience": profile.experience.to_owned(),
        "skills": profile.skills.to_owned(),
        "date_updated": date,
    });

    match db.create_profile(&id, profile.into_inner(), date).await {
        Ok(_result) => {
          
            HttpResponse::Created().json(created)
        },
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

/// # Change a user profile
/// Follows RFC 6902
/// 
/// https://tools.ietf.org/html/rfc6902
/// ## Request body:
/// ```
/// {
///    "op": "add" | "update" | "remove",
///    "path": <field>,
///    "value": <new value>
/// }
/// ```
/// ## Response:  (if successful)
/// ```
/// 204 No Content
/// ```
#[patch("/{id}")]
pub async fn change_profile(db: Data<DatabaseRepository>, path: Path<String>, profile: Json<ProfilePatch>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let date = chrono::Utc::now().timestamp();
    let target = profile.path.to_owned();
    let value = profile.value.to_owned();

    match profile.op.as_str() {
        "add" => {
            return match db.add_profile_field(&id, target, value, date).await {
                // http 204 No Content
                Ok(_result) => HttpResponse::NoContent().finish(),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            };
        },
        "update" => {
            return match db.update_profile_field(&id, target, value, date).await {
                Ok(_result) => HttpResponse::NoContent().finish(),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            };
        },
        "remove" => {
            return match db.remove_profile_field(&id, target, value, date).await {
                Ok(_result) => HttpResponse::NoContent().finish(),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            };
        },
        _ => {
            return HttpResponse::BadRequest().body("Invalid operation");
        }
    }
}