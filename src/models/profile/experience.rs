use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Experience {
    /// Field ID of the experience object
    pub field_id: Option<String>,

    /// Name of the experience (e.g. Product Manager)
    pub name: String,

    /// Type of experience (e.g. Work, Volunteer, Personal, Other)
    #[serde(rename = "type")]
    pub type_: ExperienceType,

    /// Name of company, organization, etc.
    pub at: String,

    /// Experience is current or not
    pub current: bool,

    /// Description of the experience
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExperienceType {
    /// Experience is work
    Work,

    /// Experience is volunteer
    Volunteer,

    /// Experience is personal
    Personal,

    /// Experience is other
    Other,
}
