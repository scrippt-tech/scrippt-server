use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Skills {
    /// Field ID of the skills object
    pub field_id: Option<String>,

    /// Name of the skill
    pub skill: String,
}
