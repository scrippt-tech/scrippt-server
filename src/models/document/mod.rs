pub mod traits;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    /// Field ID of the document object
    pub field_id: Option<String>,

    /// Title of the document
    pub title: String,

    /// Prompt of the document
    pub prompt: String,

    /// Content of the document
    pub content: String,

    /// Rating of the document
    pub rating: Rating,

    /// Date the document was created
    pub date_created: Option<i64>,

    /// Date the document was last updated
    pub date_updated: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Rating {
    /// Rating is none
    /// This is the default value
    None,

    /// Rating is good
    Good,

    /// Rating is bad
    Bad,
}
