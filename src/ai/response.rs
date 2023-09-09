use async_openai::types::Usage;

/// Response from the AI API.
pub struct AIClientResponse {
    pub text: String,
    pub usage: Option<Usage>,
}

impl AIClientResponse {
    pub fn new(text: String, usage: Option<Usage>) -> Self {
        Self { text, usage }
    }
}
