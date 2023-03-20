use crate::models::profile::Profile;
use async_openai::{
    error::OpenAIError,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, CreateChatCompletionResponse, Role},
    Client,
};

/// Function to generate a chat completion request. Uses the async_openai
/// crate to generate a request to the OpenAI API.
///
/// The model is set to gpt-3.5-turbo, which is the most powerful model available.
/// This function preps the model by telling it the context of the chat completion,
/// providing the user's profile information, and then providing the prompt.
///
/// # Example
/// ```rust
/// use server::models::profile::Profile;
/// use server::utils::openai::generate_request;
///
/// let response = generate_request(
///    "Why do you want to work at this company?".to_string(),
///    Profile::default(),
///    "I am a hard worker and I am very passionate about this job.".to_string(),
/// ).await;
///
/// let response = response.unwrap(); // This assumes that the request was successful
///
/// for choice in response.choices {
///    println!("Response: {:#?}", choice.message.content);
/// }
/// ```
pub async fn generate_request(prompt: String, profile: Profile, additional: String) -> Result<CreateChatCompletionResponse, OpenAIError> {
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
                .content(format!("This is additional information about yourself: {}", additional))
                .build()?,
            ChatCompletionRequestMessageArgs::default().role(Role::System).content(prompt).build()?,
        ])
        .build()?;
    log::debug!("Request: {:#?}", request);

    Ok(client.chat().create(request).await?)
}
