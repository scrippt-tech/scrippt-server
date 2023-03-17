use core::fmt;

use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use async_openai::Client;
use serde::{Deserialize, Serialize};

use crate::auth::user_auth::AuthorizationService;
use crate::models::profile::Profile;
use crate::repository::database::DatabaseRepository;

#[derive(Debug, Serialize, Deserialize)]
pub struct Highlights {
    pub prompt: String,
    pub profile: Profile,
    pub additional: String,
    pub job_url: String,
}

fn generate_prompt(prompt: String, profile: Profile, additional: String) -> String {
    let mut prompt = prompt;
    prompt.push_str(&format!(
        r#"
        Generate an answer to the following prompt: {}\nUse the following information about candidate: Education: {:#?}\nExperience: {:#?}Skills: {:#?}\nAdditional information: {:#?}.\nAnswer the prompt in a way that highlights the candidate's strengths and experience."#,
        prompt, profile.education, profile.experience, profile.skills, additional
    ));
    prompt
}

#[post("")]
pub async fn generate_openai(
    db: Data<DatabaseRepository>,
    data: Json<Highlights>,
    auth: AuthorizationService,
) -> HttpResponse {
    let id = auth.id;
    let client = Client::new();
    let prompt = generate_prompt(
        data.prompt.clone(),
        data.profile.clone(),
        data.additional.clone(),
    );
    let response = client.completions().create(prompt).await;

    match response {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::debug!("Error: {:#?}", e);
            HttpResponse::BadRequest().body("Error")
        }
    }
}
