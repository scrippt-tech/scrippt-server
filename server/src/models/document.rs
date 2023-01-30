use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub prompt: String,
    pub rating: Option<i8>,
}
