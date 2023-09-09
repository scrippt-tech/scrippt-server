use actix_web::{post, web::Json, HttpResponse};

use serde::{Deserialize, Serialize};

use crate::ai::ai;
use crate::auth::user_auth::AuthorizationService;
use crate::handlers::types::ErrorResponse;
use crate::models::profile::profile::Profile;

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

impl GenerateResponse {
    pub fn new(response: String) -> Self {
        Self { response }
    }
}

#[post("/response")]
pub async fn generate_openai(data: Json<Highlights>, _auth: AuthorizationService) -> HttpResponse {
    let client = ai::AIClient::new();
    let response = client.generate_request(data.prompt.clone(), data.profile.clone(), data.additional.clone()).await;

    match response {
        Ok(response) => HttpResponse::Ok().json(GenerateResponse::new(response.text)),
        Err(e) => {
            log::error!("Error: {:#?}", e);
            HttpResponse::BadRequest().json(ErrorResponse::new("error generating response".to_string(), e.to_string()))
        }
    }
}
