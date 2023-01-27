use actix_web::{web::{Data, Json, Path}, get, post, delete, put, HttpResponse};

#[post("/create")]
pub async fn create_profile() -> HttpResponse {
    HttpResponse::Ok().body("Create profile")
}

#[get("/{id}")]
pub async fn get_profile_by_id() -> HttpResponse {
    HttpResponse::Ok().body("Get profile by id")
}

#[put("/{id}")]
pub async fn update_profile() -> HttpResponse {
    HttpResponse::Ok().body("Update profile")
}

#[delete("/{id}")]
pub async fn delete_profile() -> HttpResponse {
    HttpResponse::Ok().body("Delete profile")
}