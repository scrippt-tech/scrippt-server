use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

use crate::auth::user_auth::AuthorizationService;
use crate::repository::database::DatabaseRepository;

#[derive(Debug, Serialize, Deserialize)]
pub struct Generate {
    pub prompt: String,
    pub skills: Vec<String>,
    pub experience: Vec<String>,
    pub additional: String,
}

/*
   User sends a prompt to the server, as well as the skills, and experience they wish to highlight when generating a response to the prompt.

   Request body structure:
   {
       "prompt": "This is a test prompt",
       "skills": [id, id, id]
       "experience": [(target, id), (target, id), (target, id)]
}

*/

#[post("")]
pub async fn generate_openai(
    db: Data<DatabaseRepository>,
    data: Json<Generate>,
    auth: AuthorizationService,
) -> HttpResponse {
    let id = auth.id;
    let relevant_data = db
        .get_profile_data(&id, &data.skills, &data.experience)
        .await;

    match relevant_data {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            log::error!("Error: {:#?}", e);
            HttpResponse::InternalServerError().body("Error getting profile data")
        }
    }
}
