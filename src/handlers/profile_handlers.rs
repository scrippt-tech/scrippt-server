use crate::ai::ai;
use crate::handlers::types::ErrorResponse;
use crate::models::profile::profile::ProfileValue;
use crate::repository::database::DatabaseRepository;
use crate::{auth::user_auth::AuthorizationService, models::profile::profile::Profile};
use actix_web::{
    patch, post,
    web::{BytesMut, Data, Json, Payload},
    HttpResponse,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProfilePatch {
    pub op: String,
    pub target: String,
    pub value: ProfileValue,
}

// MAX_PROFILE_FIELD
const MAX_PROFILE_FIELD: usize = 5;

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
                if maxed_profile_field(&db, &id, &target).await.is_err() {
                    return HttpResponse::BadRequest().body(format!("Max fields for {} reached. Remove to add a new one", target));
                }
                match db.add_profile_field(&id, target, value, date).await {
                    Ok(_result) => continue,
                    Err(e) => {
                        return HttpResponse::InternalServerError().json(ErrorResponse::new("error adding profile field".to_string(), e.to_string()))
                    }
                };
            }
            "update" => {
                match db.update_profile_field(&id, target, value, date).await {
                    Ok(_result) => continue,
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .json(ErrorResponse::new("error updating profile field".to_string(), e.to_string()))
                    }
                };
            }
            "remove" => {
                match db.remove_profile_field(&id, target, value, date).await {
                    Ok(_result) => continue,
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .json(ErrorResponse::new("error removing profile field".to_string(), e.to_string()))
                    }
                };
            }
            _ => {
                log::debug!("Invalid operation");
                return HttpResponse::BadRequest().json(ErrorResponse::new("Invalid operation".to_string(), "".to_string()));
            }
        }
    }
    match db.get_account(&id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse::new("error getting account".to_string(), e.to_string())),
    }
}

#[post("/resume")]
pub async fn profile_from_resume(db: Data<DatabaseRepository>, mut payload: Payload, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    let mut bytes = BytesMut::new();
    while let Some(item) = payload.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }

    let resume_text = match pdf_extract::extract_text_from_mem(&bytes) {
        Ok(text) => text,
        Err(e) => {
            log::error!("Error: {:#?}", e);
            return HttpResponse::BadRequest().json(ErrorResponse::new("error parsing resume".to_string(), e.to_string()));
        }
    };

    log::debug!("Resume: {:#?}", resume_text);
    // return HttpResponse::Ok().json("temp");
    let client = ai::AIClient::new();

    match client.process_resume(resume_text).await {
        Ok(response) => {
            let content = response.text;
            // pretty print content
            log::debug!("Response: {:#?}", content);
            let profile = Profile::from_json(&content).unwrap();
            match db.update_profile(&id, profile).await {
                Ok(_) => match db.get_account(&id).await {
                    Ok(user) => HttpResponse::Ok().json(user),
                    Err(e) => {
                        log::error!("Error: {:#?}", e);
                        HttpResponse::InternalServerError().json(ErrorResponse::new("error getting account".to_string(), e.to_string()))
                    }
                },
                Err(e) => {
                    log::error!("Error: {:#?}", e);
                    HttpResponse::InternalServerError().json(ErrorResponse::new(
                        "Error parsing resume. Please make sure your resume is formatted correctly and try again.".to_string(),
                        e.to_string(),
                    ))
                }
            }
        }
        Err(e) => {
            log::error!("Error: {:#?}", e);
            HttpResponse::BadRequest().json(ErrorResponse::new("error parsing resume".to_string(), e.to_string()))
        }
    }
}

async fn maxed_profile_field(db: &DatabaseRepository, id: &str, field: &str) -> Result<bool, String> {
    let profile = db.get_account(id).await.unwrap().profile;
    match field {
        "experience" => {
            if profile.experience.len() >= MAX_PROFILE_FIELD {
                return Err("Max experience fields reached".to_owned());
            }
        }
        "education" => {
            if profile.education.len() >= MAX_PROFILE_FIELD {
                return Err("Max education fields reached".to_owned());
            }
        }
        "skills" => {
            if profile.skills.len() >= MAX_PROFILE_FIELD {
                return Err("Max skills fields reached".to_owned());
            }
        }
        _ => {
            return Err("Invalid field".to_owned());
        }
    }
    Ok(true)
}
