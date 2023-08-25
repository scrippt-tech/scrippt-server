use serde::{Deserialize, Serialize};

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
