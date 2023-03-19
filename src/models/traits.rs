use crate::models::profile::*;
use mongodb::bson::oid::ObjectId;

/// Trait that allows us to update the field_id of a ProfileValue
pub trait UpdateFieldId {
    fn update_field_id(&mut self, new_id: Option<String>);
}

impl UpdateFieldId for ProfileValue {
    fn update_field_id(&mut self, new_id: Option<String>) {
        match self {
            ProfileValue::Experience(exp) => {
                exp.field_id = new_id;
            }
            ProfileValue::Education(edu) => {
                edu.field_id = new_id;
            }
            ProfileValue::Skills(skill) => {
                skill.field_id = new_id;
            }
            ProfileValue::FieldId(field_id) => {
                *field_id = new_id.unwrap();
            }
        }
    }
}

pub trait GetFieldId {
    fn get_field_id(&self) -> Option<String>;
}

impl GetFieldId for ProfileValue {
    fn get_field_id(&self) -> Option<String> {
        match self {
            ProfileValue::Experience(exp) => exp.field_id.clone(),
            ProfileValue::Education(edu) => edu.field_id.clone(),
            ProfileValue::Skills(skill) => skill.field_id.clone(),
            ProfileValue::FieldId(field_id) => Some(field_id.clone()),
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
