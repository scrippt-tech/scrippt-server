use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

/// Profile models
#[derive(Clone)]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Profile {
    pub education: Vec<Education>,
    pub experience: Vec<Experience>,
    pub skills: Vec<Skill>,
    pub date_updated: Option<i64>,
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
    pub index: Option<ObjectId>,
    pub name: String,
    pub type_: i32,
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
    pub index: Option<ObjectId>,
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
    pub index: Option<ObjectId>,
    pub skill: String,
    pub level: String,
}