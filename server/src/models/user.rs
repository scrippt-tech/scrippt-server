use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::models::profile::ProfileInfo;
use crate::models::document::DocumentInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub profile: Option<ProfileInfo>,
    pub documents: Option<Vec<DocumentInfo>>,
    pub date_created: Option<i64>,
    pub date_updated: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub id: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub profile: ProfileInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdate {
    pub name: String,
    pub email: String,
    pub date_updated: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
}