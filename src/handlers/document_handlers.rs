use actix_web::{
    delete, put,
    web::{Data, Json, Path},
    HttpResponse,
};

use serde::{Deserialize, Serialize};

use crate::{
    auth::user_auth::AuthorizationService,
    models::document::document::{Document, Rating},
    repository::database::DatabaseRepository,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentRequest {
    pub field_id: Option<String>,
    pub title: String,
    pub prompt: String,
    pub content: String,
}

#[put("")]
pub async fn create_update_document(db: Data<DatabaseRepository>, doc: Json<DocumentRequest>, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    log::debug!("CREATING DOCUMENT: ID: {:#?}", id);
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    log::debug!("Document: {:#?}", doc);
    if doc.field_id.is_some() && db.document_exists(doc.field_id.as_ref().unwrap()).await.unwrap() {
        match db.update_document(&id, doc.field_id.as_ref().unwrap(), &doc.title, &doc.content, None).await {
            Ok(result) => {
                if result.matched_count == 1 {
                    match db.get_account(&id).await {
                        Ok(user) => HttpResponse::Ok().json(user),
                        Err(e) => {
                            log::error!("Error: {:#?}", e);
                            HttpResponse::InternalServerError().json(e.to_string())
                        }
                    }
                } else {
                    HttpResponse::InternalServerError().body("Error")
                }
            }
            Err(e) => {
                log::error!("Error: {:#?}", e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        }
    } else {
        let new_doc = Document {
            field_id: None,
            title: doc.title.to_owned(),
            prompt: doc.prompt.to_owned(),
            content: doc.content.to_owned(),
            rating: Rating::None,
            date_created: Some(chrono::Utc::now().timestamp()),
            date_updated: Some(chrono::Utc::now().timestamp()),
        };

        match db.add_document(&id, new_doc).await {
            Ok(result) => {
                if result.matched_count == 1 {
                    match db.get_account(&id).await {
                        Ok(user) => HttpResponse::Ok().json(user),
                        Err(e) => {
                            log::error!("Error: {:#?}", e);
                            HttpResponse::InternalServerError().json(e.to_string())
                        }
                    }
                } else {
                    HttpResponse::InternalServerError().body("Error")
                }
            }
            Err(e) => {
                log::error!("Error: {:#?}", e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        }
    }
}

#[delete("{field_id}")]
pub async fn delete_document(db: Data<DatabaseRepository>, path: Path<String>, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }
    let field_id = path.into_inner();
    match db.delete_document(&id, &field_id).await {
        Ok(result) => {
            if result.matched_count == 1 {
                match db.get_account(&id).await {
                    Ok(user) => HttpResponse::Ok().json(user),
                    Err(e) => {
                        log::error!("Error: {:#?}", e);
                        HttpResponse::InternalServerError().json(e.to_string())
                    }
                }
            } else {
                HttpResponse::InternalServerError().body("Error")
            }
        }
        Err(e) => {
            log::error!("Error: {:#?}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}
