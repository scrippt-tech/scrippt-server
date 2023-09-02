use crate::models::profile::profile::Profile;
use async_openai::{
    error::OpenAIError,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, CreateChatCompletionResponse, Role},
    Client,
};
use pdf_extract;

pub struct AIClient {
    client: Client,
    model: String,
}

impl Default for AIClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
            model: "gpt-3.5-turbo".to_string(),
        }
    }
}

impl AIClient {
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the model to use for the request.
    /// Ex. gpt-3.5-turbo, davinci, etc.
    pub fn with_model<S: From<String>>(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    /// Generate a chat completion request. Uses the async_openai
    /// crate to generate a request to the OpenAI API.
    ///
    /// The model is set to gpt-3.5-turbo.
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
    pub async fn generate_request(self, prompt: String, profile: Profile, additional: String) -> Result<CreateChatCompletionResponse, OpenAIError> {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1024u16)
            .model(self.model)
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

        self.client.chat().create(request).await
    }

    /// Process a resume and extract the information from it.
    pub async fn process_resume(self, resume_as_bytes: Vec<u8>) -> Result<CreateChatCompletionResponse, OpenAIError> {
        let resume_text = pdf_extract::extract_text_from_mem(&resume_as_bytes).unwrap();
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1024u16)
            .model(self.model)
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content(
                        "You are an extremely accurate resume parser. When you get a resume, you need to clean the text and extract the following information in the format specified below:
                        {
                            \"education\": [
                                {
                                    \"school\": <string>,
                                    \"degree\": <string>,
                                    \"field_of_study\": <string>,
                                    \"current\": <bool>,
                                    \"description\": <string>,
                                }
                            ],
                            \"experience\": [
                                {
                                    \"name\": <string>,
                                    \"type\": 'work' | 'volunteer' | 'personal' | 'other',
                                    \"at\": <string>,
                                    \"current\": <bool>,
                                    \"description\": <string>,
                                }
                            ],
                            \"skills\": [
                                {
                                    \"skill\": <string> [THIS IS A SINGLE WORD],
                                }
                            ],
                        }
                        ").build()?,
                ChatCompletionRequestMessageArgs::default().role(Role::System).content("You can consider sections named 'Projects' as experience.").build()?,
                ChatCompletionRequestMessageArgs::default().role(Role::System).content(resume_text).build()?,
                ChatCompletionRequestMessageArgs::default().role(Role::System).content("Extracted information:").build()?,
            ])
            .build()?;
        log::debug!("Request: {:#?}", request);

        self.client.chat().create(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_request() {
        let client = AIClient::new();
        let response = client
            .generate_request(
                "Why do you want to work at this company?".to_string(),
                Profile::default(),
                "I am a hard worker and I am very passionate about this job.".to_string(),
            )
            .await;
        assert!(response.is_ok());
    }

    // #[tokio::test]
    // async fn test_process_resume() {
    //     let client = AIClient::new();
    //     let response = client.process_resume(include_bytes!("../../tests/resume.pdf").to_vec()).await;q
    // }
}
