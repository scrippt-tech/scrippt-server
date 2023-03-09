use actix_web::{
    patch,
    web::{Data, Json},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::auth::user_auth::AuthorizationService;
use crate::repository::database::DatabaseRepository;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProfilePatch {
    pub op: String,
    pub path: String,
    pub value: serde_json::Value,
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
/// 200 OK
///
/// Body:  (if successful)
/// {
///     "experience": Array,
///     "education": Array,
///     "skills": Array,
///     "date_updated": Int,
/// }
/// ```
#[patch("")]
pub async fn change_profile(
    db: Data<DatabaseRepository>,
    profile: Json<Vec<ProfilePatch>>,
    auth: AuthorizationService,
) -> HttpResponse {
    let id = auth.id;
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    for change in profile.iter() {
        let target = change.path.to_owned();
        let value = change.value.to_owned();
        let date = chrono::Utc::now().timestamp();
        match change.op.as_str() {
            "add" => {
                match db.add_profile_field(&id, target, value, date).await {
                    Ok(_result) => continue,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };
            }
            "update" => {
                match db.update_profile_field(&id, target, value, date).await {
                    Ok(_result) => continue,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };
            }
            "remove" => {
                match db.remove_profile_field(&id, target, value, date).await {
                    Ok(_result) => continue,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };
            }
            _ => {
                return HttpResponse::BadRequest().body("Invalid operation");
            }
        }
    }
    let new_profile = db.get_profile(&id).await.unwrap();
    HttpResponse::Ok().json(new_profile)
}
