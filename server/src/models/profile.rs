use serde::{Serialize, Deserialize};
use mongodb::bson::{self, Bson};
use bson::to_bson;
use serde_json;

/// Database models
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub account_id: String,
    pub profile: ProfileInfo,
    pub date_updated: Option<i64>,
}

/// Profile models
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub education: Vec<Education>,
    pub experience: Vec<Experience>,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExperienceType {
    Work,
    Volunteer,
    Personal,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Experience {
    pub name: String,
    pub type_: ExperienceType,
    pub title: String,
    pub location: String,
    pub from: String,
    pub to: String,
    pub current: bool,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Education {
    pub school: String,
    pub degree: String,
    pub field_of_study: String,
    pub from: String,
    pub to: String,
    pub current: bool,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skill {
    pub skill: String,
    pub level: String,
}


/// Request - Response models
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileResponse {
    pub account_id: String,
    pub profile: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileUpdate {
    pub profile: serde_json::Value,
}

/// Implementations
impl std::convert::From<ProfileInfo> for Bson {
    fn from(profile: ProfileInfo) -> Self {
        Bson::Document(to_bson(&profile).unwrap().as_document().unwrap().clone())
    }
}