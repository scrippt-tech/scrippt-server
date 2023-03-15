use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub title: String, // Unique
    pub prompt: String,
    pub content: String,
    pub rating: Option<i32>,
    pub date_created: Option<i64>,
    pub date_updated: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentRequest {
    pub title: String,
    pub prompt: String,
    pub content: String,
    pub op: String,
}
