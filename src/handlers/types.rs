use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// The error message.
    pub message: String,
}

impl ErrorResponse {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

// Start of account handler types

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    /// The id of the user.
    pub id: String,

    /// The JWT token used for authentication.
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountPatch {
    /// The path (name) to the field to update.
    pub path: String,

    /// The value to update the field to.
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    /// The email of the user.
    pub email: String,

    /// The password of the user.
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalAccountQuery {
    /// The external token id of the user (e.g. Google OAuth).
    pub token_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationCodeQuery {
    /// The name of the user to use in the verification email.
    pub name: String,

    /// The email of the user to send the verification code to.
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationQuery {
    /// The email of the user to verify.
    pub email: String,

    /// The verification code sent to the user.
    pub code: String,
}

// end of account handler types
