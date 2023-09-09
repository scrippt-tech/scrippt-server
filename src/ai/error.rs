use async_openai::error::OpenAIError;

/// Error type for the AI client
/// Contains all possible errors that can occur
/// when using the AI client.
#[derive(Debug)]
pub enum AIClientError {
    OpenAIError(OpenAIError),
}

/// Get the error from the OpenAI API
impl From<OpenAIError> for AIClientError {
    fn from(error: OpenAIError) -> Self {
        Self::OpenAIError(error)
    }
}

/// Convert the error to a string
impl ToString for AIClientError {
    fn to_string(&self) -> String {
        match self {
            Self::OpenAIError(e) => e.to_string(),
        }
    }
}

impl AIClientError {
    pub fn openai(error: OpenAIError) -> Self {
        Self::OpenAIError(error)
    }
}
