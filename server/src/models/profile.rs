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
    pub type_: ExperienceType, // Do we care about this?
    pub at: String,
    pub current: bool,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Education {
    pub field_id: Option<ObjectId>,
    pub school: String,
    pub degree: String,
    pub field_of_study: String,
    pub current: bool,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    pub field_id: Option<ObjectId>,
    pub skill: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProfileValue {
    Education(Education),
    Experience(Experience),
    Skill(Skill),
}

/// Trait that allows us to update the field_id of a ProfileValue
pub trait UpdateFieldId {
    fn update_field_id(&mut self, new_id: Option<ObjectId>);
}

impl UpdateFieldId for Experience {
    fn update_field_id(&mut self, new_id: Option<ObjectId>) {
        self.field_id = new_id;
    }
}

impl UpdateFieldId for Education {
    fn update_field_id(&mut self, new_id: Option<ObjectId>) {
        self.field_id = new_id;
    }
}

impl UpdateFieldId for Skill {
    fn update_field_id(&mut self, new_id: Option<ObjectId>) {
        self.field_id = new_id;
    }
}

impl UpdateFieldId for ProfileValue {
    fn update_field_id(&mut self, new_id: Option<ObjectId>) {
        match self {
            ProfileValue::Experience(exp) => exp.update_field_id(new_id),
            ProfileValue::Education(edu) => edu.update_field_id(new_id),
            ProfileValue::Skill(skill) => skill.update_field_id(new_id),
        }
    }
}

/// This is a trait that allows us to get the field_id of a ProfileValue
pub trait GetFieldId {
    fn get_field_id(&self) -> Option<ObjectId>;
}

impl GetFieldId for Experience {
    fn get_field_id(&self) -> Option<ObjectId> {
        self.field_id
    }
}

impl GetFieldId for Education {
    fn get_field_id(&self) -> Option<ObjectId> {
        self.field_id
    }
}

impl GetFieldId for Skill {
    fn get_field_id(&self) -> Option<ObjectId> {
        self.field_id
    }
}

impl GetFieldId for ProfileValue {
    fn get_field_id(&self) -> Option<ObjectId> {
        match self {
            ProfileValue::Experience(exp) => exp.get_field_id(),
            ProfileValue::Education(edu) => edu.get_field_id(),
            ProfileValue::Skill(skill) => skill.get_field_id(),
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

impl Default for Skill {
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
        Self::Experience(Experience::default())
    }
}
