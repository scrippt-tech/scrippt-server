use actix_web::web::Data;
use actix_web::{post, web::Json, HttpResponse};

use orca::chains::chain::LLMChain;
use orca::chains::traits::Execute;
use orca::llm::openai::client::OpenAIClient;
use orca::prompt::prompt::PromptTemplate;
use orca::prompts;
use serde::{Deserialize, Serialize};

use crate::auth::user_auth::AuthorizationService;
use crate::handlers::types::ErrorResponse;
use crate::models::profile::{education::Education, experience::Experience, profile::Profile, skills::Skills};
use crate::utils::prompt::load_prompt;

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
    let prompt = load_prompt("response");
    if load_prompt("response").is_err() {
        return HttpResponse::InternalServerError().json(ErrorResponse::new("error loading prompt".to_string(), prompt.err().unwrap().to_string()));
    }

    let prompt_data = PromptData {
        experience: data.profile.experience.to_owned(),
        education: data.profile.education.to_owned(),
        skills: data.profile.skills.to_owned(),
        additional: data.additional.to_owned(),
        prompt: data.prompt.to_owned(),
    };

    let response = LLMChain::new(client.get_ref(), prompts!(("system", prompt.unwrap().as_str()))).execute(&prompt_data).await;

    match response {
        Ok(response) => HttpResponse::Ok().json(GenerateResponse::new(response)),
        Err(e) => {
            log::error!("Error: {:#?}", e);
            HttpResponse::BadRequest().json(ErrorResponse::new("".to_string(), "Error generating response.".to_string()))
        }
    }
}
