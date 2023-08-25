use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::models::document::document::Document;
use crate::models::profile::profile::Profile;

#[derive(Debug, Serialize, Deserialize)]
/// A struct representing a user.
pub struct User {
    /// The unique identifier for the user. Serialized as "_id" in JSON.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The name of the user.
    pub name: String,

    /// The email address of the user.
    pub email: String,

    /// The password of the user. This field is optional.
    pub password: Option<String>,

    /// An external identifier associated with the user.
    pub external_id: Option<String>,

    /// The external provider associated with the user.
    pub external_provider: Option<String>,

    /// The user's profile information. This field is optional.
    pub profile: Option<Profile>,

    /// A list of document information associated with the user. This field is optional.
    pub documents: Option<Vec<Document>>,

    /// The timestamp indicating when the user was created. This field is optional.
    pub date_created: Option<i64>,

    /// The timestamp indicating when the user was last updated. This field is optional.
    pub date_updated: Option<i64>,
}
