use mongodb::bson::oid::ObjectId;
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

/// Trait that allows us to update the field_id of a ProfileValue
pub trait UpdateFieldId {
    fn update_field_id(&mut self, new_id: Option<String>);
}

impl UpdateFieldId for Experience {
    fn update_field_id(&mut self, new_id: Option<String>) {
        self.field_id = new_id;
    }
}

impl UpdateFieldId for Education {
    fn update_field_id(&mut self, new_id: Option<String>) {
        self.field_id = new_id;
    }
}

impl UpdateFieldId for Skills {
    fn update_field_id(&mut self, new_id: Option<String>) {
        self.field_id = new_id;
    }
}

impl UpdateFieldId for String {
    fn update_field_id(&mut self, new_id: Option<String>) {
        if let Some(id) = new_id {
            self.replace_range(.., &id);
        }
    }
}

impl UpdateFieldId for ProfileValue {
    fn update_field_id(&mut self, new_id: Option<String>) {
        match self {
            ProfileValue::Experience(exp) => exp.update_field_id(new_id),
            ProfileValue::Education(edu) => edu.update_field_id(new_id),
            ProfileValue::Skills(skill) => skill.update_field_id(new_id),
            ProfileValue::FieldId(field_id) => field_id.update_field_id(new_id),
        }
    }
}

/// This is a trait that allows us to get the field_id of a ProfileValue
pub trait GetFieldId {
    fn get_field_id(&self) -> Option<String>;
}

impl GetFieldId for Experience {
    fn get_field_id(&self) -> Option<String> {
        Some(self.field_id.clone().unwrap())
    }
}

impl GetFieldId for Education {
    fn get_field_id(&self) -> Option<String> {
        Some(self.field_id.clone().unwrap())
    }
}

impl GetFieldId for Skills {
    fn get_field_id(&self) -> Option<String> {
        Some(self.field_id.clone().unwrap())
    }
}

impl GetFieldId for String {
    fn get_field_id(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl GetFieldId for ProfileValue {
    fn get_field_id(&self) -> Option<String> {
        match self {
            ProfileValue::Experience(exp) => exp.get_field_id(),
            ProfileValue::Education(edu) => edu.get_field_id(),
            ProfileValue::Skills(skill) => skill.get_field_id(),
            ProfileValue::FieldId(field_id) => field_id.get_field_id(),
        }
    }
}

// Default implementations

impl Default for Experience {
    fn default() -> Self {
        Self {
            field_id: None,
            name: String::new(),
            type_: ExperienceType::Work,
            at: String::new(),
            current: false,
            description: String::new(),
        }
    }
}

impl Default for Education {
    fn default() -> Self {
        Self {
            field_id: None,
            school: String::new(),
            degree: String::new(),
            field_of_study: String::new(),
            current: false,
            description: String::new(),
        }
    }
}

impl Default for Skills {
    fn default() -> Self {
        Self {
            field_id: None,
            skill: String::new(),
        }
    }
}

impl Default for ExperienceType {
    fn default() -> Self {
        Self::Work
    }
}

impl Default for ProfileValue {
    fn default() -> Self {
        Self::FieldId(ObjectId::new().to_hex())
    }
}
