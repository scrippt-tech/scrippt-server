use super::{education::Education, experience::Experience, skills::Skills};
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

/// Profile value enum used to deserialize a profile field
/// in which the type of the field is unknown.
/// TODO: Switch to external tagging
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum ProfileValue {
    /// Field ID of the profile object
    FieldId(String),

    /// Name of the experience (e.g. Product Manager)
    Experience(Experience),

    /// Name of the school
    Education(Education),

    /// Name of the skill
    Skills(Skills),
}
