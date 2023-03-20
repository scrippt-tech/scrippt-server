use actix_web::{
    delete, post, put,
    web::{Data, Json},
    HttpResponse,
};

use serde::{Deserialize, Serialize};

use crate::{auth::user_auth::AuthorizationService, models::document::DocumentInfo, repository::database::DatabaseRepository};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentRequest {
    pub title: String,
    pub prompt: String,
    pub content: String,
}

#[post("")]
pub async fn create_document(db: Data<DatabaseRepository>, doc: Json<DocumentRequest>, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    if db.document_exists(&doc.title).await.unwrap() {
        return HttpResponse::Conflict().body("Document already exists");
    }

    let new_doc = DocumentInfo {
        title: doc.title.to_owned(),
        prompt: doc.prompt.to_owned(),
        content: doc.content.to_owned(),
        rating: None,
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

#[put("")]
pub async fn update_document(db: Data<DatabaseRepository>, doc: Json<DocumentRequest>, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    match db.update_document(&id, &doc.title, &doc.content, None).await {
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

#[delete("")]
pub async fn delete_document(db: Data<DatabaseRepository>, doc: Json<DocumentRequest>, auth: AuthorizationService) -> HttpResponse {
    let id = auth.id;
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    match db.delete_document(&id, &doc.title).await {
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
