use crate::models::profile::profile::Profile;
use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};

use super::error::AIClientError;
use super::response::AIClientResponse;

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
    /// # Example
    /// ```rust
    ///
    /// use server::models::profile::profile::Profile;
    /// use server::ai::ai::AIClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    /// let client = AIClient::new();
    /// let response = client.generate_request(
    ///    "Why do you want to work at this company?".to_string(),
    /// Profile::default(),
    ///    "I am a hard worker and I am very passionate about this job.".to_string())
    ///    .await
    ///    .unwrap(); // This assumes that the request was successful
    ///
    /// println!("Response: {:#?}", response.text);
    /// }
    /// ```
    pub async fn generate_request(self, prompt: String, profile: Profile, additional: String) -> Result<AIClientResponse, AIClientError> {
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

        match self.client.chat().create(request).await {
            Ok(res) => Ok(AIClientResponse::new(res.choices[0].to_owned().message.content, res.usage)),
            Err(e) => Err(AIClientError::openai(e)),
        }
    }

    /// Process a resume and extract the information from it.
    ///
    /// # Example
    /// ```rust
    /// use server::ai::ai::AIClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///    let client = AIClient::new();
    ///    let response = client.process_resume(include_bytes!("../../tests/sample-resume.pdf").to_vec()).await;
    ///    assert!(response.is_ok());
    /// }
    /// ```
    pub async fn process_resume(self, resume_text: String) -> Result<AIClientResponse, AIClientError> {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1024u16)
            .model(self.model)
            .temperature(0.2)
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content(
                        "You are an extremely accurate resume parser. When you get a resume, you need to clean the text and extract the following information in the following format:
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
                        }
                        ").build()?,
                ChatCompletionRequestMessageArgs::default().role(Role::System).content("None of the values can be null.").build()?,
                ChatCompletionRequestMessageArgs::default().role(Role::System).content(resume_text).build()?,
                ChatCompletionRequestMessageArgs::default().role(Role::System).content("Extracted information:").build()?,
            ])
            .build()?;
        log::debug!("Request: {:#?}", request);

        match self.client.chat().create(request).await {
            Ok(res) => Ok(AIClientResponse::new(res.choices[0].to_owned().message.content, res.usage)),
            Err(e) => Err(AIClientError::openai(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdf_extract::extract_text;

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

    #[tokio::test]
    async fn test_process_resume() {
        let client = AIClient::new();
        let resume_text = extract_text("../../tests/sample-resume.pdf").unwrap();
        let response = client.process_resume(resume_text).await;
        assert!(response.is_ok());
    }
}
