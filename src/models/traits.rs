/// Traits for the models

/// Trait that allows us to update the field_id of a model
pub trait UpdateFieldId {
    fn update_field_id(&mut self, new_id: Option<String>);
}

/// Trait that allows us to get the field_id of a model
pub trait GetFieldId {
    fn get_field_id(&self) -> Option<String>;
}
