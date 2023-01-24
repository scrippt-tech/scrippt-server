use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
// add date created and date updated



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