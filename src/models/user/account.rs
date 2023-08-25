use crate::models::document::document::Document;
use crate::models::profile::profile::Profile;
use serde::{Deserialize, Serialize};

/// A struct representing an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    /// The unique identifier for the account.
    pub id: String,

    /// The name associated with the account.
    pub name: String,

    /// The email address associated with the account.
    pub email: String,

    /// The profile information of the account.
    pub profile: Profile,

    /// A list of document information associated with the account.
    pub documents: Vec<Document>,
}
