use super::{document::Document, document::Rating};
use crate::models::traits::{GetFieldId, UpdateFieldId};
use bson::Bson;

impl From<Rating> for Bson {
    fn from(rating: Rating) -> Self {
        Bson::String(match rating {
            Rating::None => "none".to_string(),
            Rating::Good => "good".to_string(),
            Rating::Bad => "bad".to_string(),
        })
    }
}

impl UpdateFieldId for Document {
    fn update_field_id(&mut self, new_id: Option<String>) {
        self.field_id = new_id;
    }
}

impl GetFieldId for Document {
    fn get_field_id(&self) -> Option<String> {
        self.field_id.clone()
    }
}
