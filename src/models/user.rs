use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::models::document::DocumentInfo;
use crate::models::profile::Profile;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub external_id: Option<String>,
    pub external_provider: Option<String>,
    pub profile: Option<Profile>,
    pub documents: Option<Vec<DocumentInfo>>,
    pub date_created: Option<i64>,
    pub date_updated: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub profile: Profile,
    pub documents: Vec<DocumentInfo>,
}
