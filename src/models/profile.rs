use serde::{Serialize, Deserialize};
use mongodb::bson::{self, Bson};
use bson::to_bson;

/// Profile models
#[derive(Clone)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub education: Vec<Education>,
    pub experience: Vec<Experience>,
    pub skills: Vec<Skill>,
}

#[derive(Clone)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ExperienceType {
    Work,
    Volunteer,
    Personal,
    Other,
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Skill {
    pub skill: String,
    pub level: String,
}

/// Implementations
impl std::convert::From<ProfileInfo> for Bson {
    fn from(profile: ProfileInfo) -> Self {
        Bson::Document(to_bson(&profile).unwrap().as_document().unwrap().clone())
    }
}