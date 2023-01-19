use actix_web::{get, post, delete, put, Responder};

#[post("")]
pub async fn create_profile() -> impl Responder {
    "Create profile"
}

#[get("/{id}")]
pub async fn get_profile_by_id() -> impl Responder {
    "Get profile by id"
}

#[put("/{id}")]
pub async fn update_profile() -> impl Responder {
    "Update profile"
}

#[delete("/{id}")]
pub async fn delete_profile() -> impl Responder {
    "Delete profile"
}