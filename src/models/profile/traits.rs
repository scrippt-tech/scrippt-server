use crate::models::profile::{education::Education, experience::Experience, experience::ExperienceType, profile::ProfileValue, skills::Skills};
use crate::models::traits::{GetFieldId, UpdateFieldId};
use bson::oid::ObjectId;

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

impl UpdateFieldId for Education {
    fn update_field_id(&mut self, new_id: Option<String>) {
        self.field_id = new_id;
    }
}

impl UpdateFieldId for Experience {
    fn update_field_id(&mut self, new_id: Option<String>) {
        self.field_id = new_id;
    }
}

impl UpdateFieldId for Skills {
    fn update_field_id(&mut self, new_id: Option<String>) {
        self.field_id = new_id;
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
