use actix_web::web::Data;
use actix_web::{post, web::Json, HttpResponse};
use serde::{Deserialize, Serialize};

use orca::chains::chain::LLMChain;
use orca::chains::Chain;
use orca::llm::openai::OpenAIClient;
use orca::prompt::prompt::PromptEngine;
use orca::prompts;

use crate::auth::user_auth::AuthorizationService;
use crate::handlers::types::ErrorResponse;
use crate::models::profile::{education::Education, experience::Experience, skills::Skills, Profile};
use crate::prompts::RESPONSE;

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

#[derive(Serialize)]
struct PromptData {
    experience: Vec<Experience>,
    education: Vec<Education>,
    skills: Vec<Skills>,
    additional: String,
    prompt: String,
}

#[post("/response")]
pub async fn generate_openai(client: Data<OpenAIClient>, data: Json<Highlights>, _auth: AuthorizationService) -> HttpResponse {
    let prompt = *RESPONSE;

    let prompt_data = PromptData {
        experience: data.profile.experience.to_owned(),
        education: data.profile.education.to_owned(),
        skills: data.profile.skills.to_owned(),
        additional: data.additional.to_owned(),
        prompt: data.prompt.to_owned(),
    };

    let mut chain = LLMChain::new(client.get_ref()).with_prompt(prompts!(("system", prompt)));
    chain.load_context(&prompt_data);
    let response = chain.execute().await;

    match response {
        Ok(response) => HttpResponse::Ok().json(GenerateResponse::new(response.content())),
        Err(e) => {
            log::error!("Error: {:#?}", e);
            HttpResponse::BadRequest().json(ErrorResponse::new("".to_string(), "Error generating response.".to_string()))
        }
    }
}
