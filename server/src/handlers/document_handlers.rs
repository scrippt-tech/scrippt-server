use actix_web::{web::{Data, Json, Path}, put, HttpResponse};

use crate::{
    repository::db::DatabaseRepository, 
    models::document::{DocumentInfo, DocumentRequest},
    auth::user_auth::AuthorizationService,
};

#[put("/{id}")]
pub async fn document(db: Data<DatabaseRepository>, path: Path<String>, doc: Json<DocumentRequest>, _auth: AuthorizationService) -> HttpResponse {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let new_doc = DocumentInfo {
        title: doc.title.to_owned(),
        prompt: doc.prompt.to_owned(),
        content: doc.content.to_owned(),
        rating: None,
        date_created: Some(chrono::Utc::now().timestamp()),
        date_updated: Some(chrono::Utc::now().timestamp()),
    };

    if doc.op == "new" {
        let date = chrono::Utc::now().timestamp();
        match db.add_document(&id, new_doc, date).await {
            Ok(_result) => HttpResponse::Ok().body("Document created"),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    } else {
        return HttpResponse::BadRequest().body("Invalid operation");
    }
}