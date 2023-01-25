use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub date_created: Option<i64>,
    pub date_updated: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResponse {
    pub id: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub email: String,
    pub exp: usize,
}