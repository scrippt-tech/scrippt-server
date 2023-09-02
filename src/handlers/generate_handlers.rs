use actix_web::{
    post,
    web::{BytesMut, Data, Json, Payload},
    HttpResponse,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::generate::ai;
use crate::models::profile::profile::Profile;
use crate::repository::database::DatabaseRepository;
use crate::{auth::user_auth::AuthorizationService, handlers::types::ErrorResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct Highlights {
    pub prompt: String,
    pub profile: Profile,
    pub additional: String,
    pub job_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub response: String,
}

#[post("/response")]
pub async fn generate_openai(data: Json<Highlights>, _auth: AuthorizationService) -> HttpResponse {
    let client = ai::AIClient::new();
    let response = client.generate_request(data.prompt.clone(), data.profile.clone(), data.additional.clone()).await;

    let mut res: Vec<GenerateResponse> = Vec::new();

    match response {
        Ok(response) => {
            for choice in response.choices {
                res.push(GenerateResponse {
                    response: choice.message.content,
                });
            }
            HttpResponse::Ok().json(res)
        }
        Err(e) => {
            log::error!("Error: {:#?}", e);
            HttpResponse::BadRequest().body("Error")
        }
    }
}

#[post("/profile")]
pub async fn profile_from_resume(db: Data<DatabaseRepository>, mut payload: Payload, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    let mut bytes = BytesMut::new();
    while let Some(item) = payload.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }

    let client = ai::AIClient::new();
    let response = client.process_resume(bytes.to_vec()).await;

    match response {
        Ok(response) => {
            let content = &response.choices[0].message.content;
            // pretty print content
            log::debug!("Response: {:#?}", content);
            let profile = Profile::from_json(content).unwrap();
            match db.update_profile(&id, profile).await {
                Ok(_) => match db.get_account(&id).await {
                    Ok(user) => HttpResponse::Ok().json(user),
                    Err(e) => {
                        log::error!("Error: {:#?}", e);
                        HttpResponse::InternalServerError().json(ErrorResponse::new(e.to_string()))
                    }
                },
                Err(e) => {
                    log::error!("Error: {:#?}", e);
                    HttpResponse::InternalServerError().json(ErrorResponse::new(
                        "Error parsing resume. Please make sure your resume is formatted correctly and try again.".to_string(),
                    ))
                }
            }
        }
        Err(e) => {
            log::error!("Error: {:#?}", e);
            HttpResponse::BadRequest().json(ErrorResponse::new(e.to_string()))
        }
    }
}
