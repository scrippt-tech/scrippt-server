use actix_web::{post, web::Json, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::auth::user_auth::AuthorizationService;
use crate::models::profile::Profile;
use crate::utils::openai::generate_request;

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

#[post("")]
pub async fn generate_openai(data: Json<Highlights>, _auth: AuthorizationService) -> HttpResponse {
    let response = generate_request(
        data.prompt.clone(),
        data.profile.clone(),
        data.additional.clone(),
    )
    .await;

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
            log::debug!("Error: {:#?}", e);
            HttpResponse::BadRequest().body("Error")
        }
    }
}
