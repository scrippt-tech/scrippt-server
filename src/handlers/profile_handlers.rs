use actix_web::{
    patch,
    web::{Data, Json},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

use crate::auth::user_auth::AuthorizationService;
use crate::models::profile::ProfileValue;
use crate::repository::database::DatabaseRepository;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProfilePatch {
    pub op: String,
    pub target: String,
    pub value: ProfileValue,
}

/// # Change a user profile
/// Follows RFC 6902
///
/// https://tools.ietf.org/html/rfc6902
/// ## Request body:
/// ```
/// {
///    "op": "add" | "update" | "remove",
///    "target": "experience" | "education" | "skills",
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
pub async fn change_profile(db: Data<DatabaseRepository>, profile: Json<Vec<ProfilePatch>>, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    if id.is_empty() {
        log::debug!("Invalid id");
        return HttpResponse::BadRequest().body("Invalid id");
    }
    log::debug!("Profile: {:#?}", profile[0].value);

    for order in profile.iter() {
        let target = order.target.to_owned();
        let value = order.value.to_owned();
        let date = chrono::Utc::now().timestamp();
        log::debug!("Target: {:#?}", target);
        log::debug!("Value: {:#?}", value);
        match order.op.as_str() {
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
                log::debug!("Invalid operation");
                return HttpResponse::BadRequest().body("Invalid operation");
            }
        }
    }
    match db.get_account(&id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}