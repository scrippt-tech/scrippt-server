use serde::{Deserialize, Serialize};

/// Profile models
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Profile {
    /// Vector of education objects
    pub education: Vec<Education>,

    /// Vector of experience objects
    pub experience: Vec<Experience>,

    /// Vector of skills objects
    pub skills: Vec<Skills>,

    /// Time of last update
    pub date_updated: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Experience {
    /// Field ID of the experience object
    pub field_id: Option<String>,

    /// Name of the experience (e.g. Product Manager)
    pub name: String,

    /// Type of experience (e.g. Work, Volunteer, Personal, Other)
    #[serde(rename = "type")]
    pub type_: ExperienceType, // Do we care about this?

    /// Name of company, organization, etc.
    pub at: String,

    /// Experience is current or not
    pub current: bool,

    /// Description of the experience
    pub description: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Education {
    /// Field ID of the education object
    pub field_id: Option<String>,

    /// Name of the school
    pub school: String,

    /// Degree of the school (e.g. Bachelor's, Master's, etc.)
    pub degree: String,

    /// Field of study of the school (e.g. Computer Science, etc.)
    pub field_of_study: String,

    /// Education is current or not
    pub current: bool,

    /// Description of the education
    pub description: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Skills {
    /// Field ID of the skills object
    pub field_id: Option<String>,

    /// Name of the skill
    pub skill: String,
}

/// Profile value enum used to deserialize a profile field
/// in which the type of the field is unknown.
/// TODO: Switch to external tagging
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum ProfileValue {
    FieldId(String),
    Experience(Experience),
    Education(Education),
    Skills(Skills),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExperienceType {
    Work,
    Volunteer,
    Personal,
    Other,
}
