use actix_web::{post, web::Json, HttpResponse};
use async_openai::{
    error::OpenAIError,
    types::{
        ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs,
        CreateChatCompletionResponse, Role,
    },
    Client,
};
use serde::{Deserialize, Serialize};

use crate::auth::user_auth::AuthorizationService;
use crate::models::profile::Profile;

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

async fn generate_request(
    prompt: String,
    profile: Profile,
    additional: String,
) -> Result<CreateChatCompletionResponse, OpenAIError> {
    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(1024u16)
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(
                    "You are a candidate who is applying for a job at a company. 
                Following you will receive some highlights about your background. 
                You will then then receive a prompt that you will need to answer. 
                Your answer should highlight your strengths and experience.",
                )
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(format!("Experience: {:#?}", profile.experience))
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(format!("Education: {:#?}", profile.education))
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(format!("Skills: {:#?}", profile.skills))
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(format!(
                    "This is additional information about yourself: {}",
                    additional
                ))
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(prompt)
                .build()?,
        ])
        .build()?;
    log::debug!("Request: {:#?}", request);

    Ok(client.chat().create(request).await?)
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
