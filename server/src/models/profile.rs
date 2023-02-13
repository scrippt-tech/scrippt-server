use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Profile models
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Profile {
    pub education: Vec<Education>,
    pub experience: Vec<Experience>,
    pub skills: Vec<Skill>,
    pub date_updated: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExperienceType {
    Work,
    Volunteer,
    Personal,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Experience {
    pub field_id: Option<ObjectId>,
    pub name: String,
    pub type_: i32,
    pub title: String,
    pub location: String,
    pub from: String,
    pub to: String,
    pub current: bool,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Education {
    pub field_id: Option<ObjectId>,
    pub school: String,
    pub degree: String,
    pub field_of_study: String,
    pub from: String,
    pub to: String,
    pub current: bool,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    pub field_id: Option<ObjectId>,
    pub skill: String,
    pub level: String,
}
