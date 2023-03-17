use serde::{Deserialize, Serialize};

/// Profile models
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Profile {
    pub education: Vec<Education>,
    pub experience: Vec<Experience>,
    pub skills: Vec<Skills>,
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
    pub field_id: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ExperienceType, // Do we care about this?
    pub at: String,
    pub current: bool,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Education {
    pub field_id: Option<String>,
    pub school: String,
    pub degree: String,
    pub field_of_study: String,
    pub current: bool,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skills {
    pub field_id: Option<String>,
    pub skill: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum ProfileValue {
    FieldId(String),
    Experience(Experience),
    Education(Education),
    Skills(Skills),
}
