use crate::handlers::types::ErrorResponse;
use crate::models::profile::profile::ProfileValue;
use crate::repository::database::DatabaseRepository;
use crate::utils::prompt::load_prompt;
use crate::{auth::user_auth::AuthorizationService, models::profile::profile::Profile};
use actix_web::{
    patch, post,
    web::{BytesMut, Data, Json, Payload},
    HttpResponse,
};
use futures::StreamExt;
use orca::chains::chain::LLMChain;
use orca::chains::traits::Execute;
use orca::llm::openai::client::OpenAIClient;
use orca::prompt::prompt::PromptTemplate;
use orca::prompts;
use orca::record::pdf::PDF;
use orca::record::spin::Spin;
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
pub async fn profile_from_resume(
    client: Data<OpenAIClient>,
    db: Data<DatabaseRepository>,
    mut payload: Payload,
    auth: AuthorizationService,
) -> HttpResponse {
    let id = auth.id;
    let mut bytes = BytesMut::new();
    while let Some(item) = payload.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }

    let record = PDF::from_buffer(bytes.to_vec(), false).spin().unwrap();
    let prompt = load_prompt("parser");
    if prompt.is_err() {
        return HttpResponse::InternalServerError().json(ErrorResponse::new(
            "Error loading parser prompt. Please try again later.".to_string(),
            prompt.err().unwrap().to_string(),
        ));
    }

    #[derive(Serialize)]
    struct Data {
        record: String,
        record_content: String,
        format: String,
    }

    // Use Orca LLM Orchestrator to parse resume
    let resume_text = LLMChain::new(client.get_ref(), prompts!(("system", prompt.unwrap().as_str())))
        .execute(&Data {
            record: "resume".to_string(),
            record_content: record.content.to_string(),
            format: FORMAT.to_string(),
        })
        .await;

    if resume_text.is_err() {
        return HttpResponse::InternalServerError().json(ErrorResponse::new(
            "Please make sure your resume is formatted correctly and try again.".to_string(),
            "Error parsing resume.".to_string(),
        ));
    }

    let profile = Profile::from_json(&resume_text.unwrap());
    if profile.is_err() {
        return HttpResponse::InternalServerError().json(ErrorResponse::new(
            "Error parsing resume.".to_string(),
            "Error reading LLM response into JSON format".to_string(),
        ));
    }
    match db.update_profile(&id, profile.unwrap()).await {
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

const FORMAT: &str = r#"
{
    \"education\": [
        {
            \"school\": <string>, // name of the school (e.g. University of California, Berkeley)
            \"degree\": <string>, // degree type (e.g. Bachelor of Science)
            \"field_of_study\": <string>, // field of study (e.g. Computer Science)
            \"current\": <bool>, // whether the candidate is currently enrolled
            \"description\": <string>, // description of the degree (e.g. GPA, honors)
        }
    ],
    \"experience\": [
        {
            \"name\": <string>, // name of the position (e.g. HR Manager)
            \"type\": 'work' | 'volunteer' | 'personal' | 'other', // type of experience
            \"at\": <string>, // name of the company (e.g. Google)
            \"current\": <bool>, // whether the candidate currently works here
            \"description\": <string>, // description of the position (e.g. responsibilities)
        }
    ],
    \"skills\": [
        {
            \"skill\": <string>, // name of the skill (e.g. Python, Javascript, Leadership, MacOS)
        }
    ],
}"#;

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
