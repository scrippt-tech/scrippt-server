use crate::models::profile::{Education, Experience, Skills};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateData {
    pub experience: Vec<Experience>,
    pub education: Vec<Education>,
    pub skills: Vec<Skills>,
}
