use bson::to_bson;
use log;
use mongodb::{
    bson::oid::ObjectId,
    bson::{doc, extjson::de::Error},
    options::ClientOptions,
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Client, Collection,
};
use serde_json;

use crate::models::document::DocumentInfo;
use crate::models::profile::Profile;
use crate::models::user::{AccountPatch, User};

pub struct DatabaseRepository {
    pub user_collection: Collection<User>,
}

impl DatabaseRepository {
    /// Initialize the repository with a MongoDB connection
    pub async fn new(uri: &str) -> Self {
        let uri = uri.to_string();
        let client_options = ClientOptions::parse(uri)
            .await
            .ok()
            .expect("Failed to parse client options");
        let client = Client::with_options(client_options);

        match client {
            Ok(client) => {
                log::info!("Connected to MongoDB");
                let db = client.database("scrippt");
                let user_collection: Collection<User> = db.collection("users");
                DatabaseRepository { user_collection }
            }
            Err(_) => {
                log::error!("Failed to connect to MongoDB");
                panic!("Panicking because of failed connection to MongoDB");
            }
        }
    }

    /// Get a user account by id
    pub async fn get_account(&self, id: &str) -> Result<User, Error> {
        let obj_id = ObjectId::parse_str(id)
            .ok()
            .expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let account_detail = self.user_collection.find_one(filter, None).await;
        match account_detail {
            Ok(Some(account)) => Ok(account),
            Ok(None) => Err(Error::DeserializationError {
                message: "Account not found".to_string(),
            }),
            Err(e) => {
                log::error!("Failed to get account {}", id);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    /// Get a user account by email
    pub async fn get_account_by_email(&self, email: &str) -> Result<User, Error> {
        let filter = doc! {"email": email};
        let account_detail = self.user_collection.find_one(filter, None).await;
        match account_detail {
            Ok(Some(account)) => Ok(account),
            Ok(None) => Err(Error::DeserializationError {
                message: "Account not found".to_string(),
            }),
            Err(e) => {
                log::error!("Failed to get account by email {}", email);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    /// Create a new account
    pub async fn create_account(&self, user: User) -> Result<InsertOneResult, Error> {
        let new_doc = User {
            id: None,
            name: user.name,
            email: user.email,
            password: user.password,
            external_id: user.external_id,
            external_provider: user.external_provider,
            profile: user.profile,
            documents: user.documents,
            date_created: user.date_created,
            date_updated: user.date_updated,
        };
        let user = self.user_collection.insert_one(new_doc, None).await;
        match user {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("Failed to create account {}", e);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    /// Update an existing account's name and email
    pub async fn update_account(
        &self,
        id: &str,
        update: AccountPatch,
    ) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id)
            .ok()
            .expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let new_doc = doc! {
            "$set": {
                update.path: update.value
            }
        };
        let updated_doc = self.user_collection.update_one(filter, new_doc, None).await;
        match updated_doc {
            Ok(result) => match result.modified_count {
                1 => Ok(result),
                _ => Err(Error::DeserializationError {
                    message: "Failed to update document".to_string(),
                }),
            },
            Err(e) => {
                log::error!("Failed to update account {}", id);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    /// Delete an existing account
    pub async fn delete_account(&self, id: &str) -> Result<DeleteResult, Error> {
        let obj_id = ObjectId::parse_str(id)
            .ok()
            .expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let account_detail = self.user_collection.delete_one(filter, None).await;
        match account_detail {
            Ok(result) => match result.deleted_count {
                1 => Ok(result),
                _ => Err(Error::DeserializationError {
                    message: "Failed to delete document".to_string(),
                }),
            },
            Err(e) => {
                log::error!("Failed to delete account {}", id);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    /// Gets a user profile field from the database by filtering on the user id
    pub async fn get_profile(&self, id: &String) -> Result<Profile, Error> {
        let obj_id = ObjectId::parse_str(id)
            .ok()
            .expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let result = self.user_collection.find_one(filter, None).await;
        match result {
            Ok(Some(user)) => user.profile.ok_or(Error::DeserializationError {
                message: "Profile not found".to_string(),
            }),
            Ok(None) => Err(Error::DeserializationError {
                message: "Profile not found".to_string(),
            }),
            Err(e) => {
                log::error!("Failed to get profile {}", id);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    /// Add profile field to the database given an id, target, value and date
    pub async fn add_profile_field(
        &self,
        id: &String,
        target: String,
        mut value: serde_json::Value,
        date: i64,
    ) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id)
            .ok()
            .expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let target = format!("profile.{}", target);
        value["field_id"] = serde_json::Value::String(ObjectId::new().to_hex());
        let update = doc! {
            "$push": {
                target: to_bson(&value).unwrap()
            },
            "$set": {
                "profile.date_updated": date,
            }
        };
        let result = self.user_collection.update_one(filter, update, None).await;
        match result {
            Ok(result) => match result.modified_count {
                1 => Ok(result),
                _ => Err(Error::DeserializationError {
                    message: "Failed to add document".to_string(),
                }),
            },
            Err(e) => {
                log::error!("Failed to add profile field for account {}", id);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    /// Update profile field
    pub async fn update_profile_field(
        &self,
        id: &String,
        target: String,
        value: serde_json::Value,
        date: i64,
    ) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id)
            .ok()
            .expect("Failed to parse object id");
        let field_id = format!("profile.{}.field_id", target); // profile.target.field_id
        let field = format!("profile.{}.$", target); // profile.target.$
        let filter = doc! {
            "_id": obj_id,
            field_id: value["field_id"].to_string()
        };
        let update = doc! {
            "$set": {
                field: to_bson(&value).unwrap(),
                "profile.date_updated": date,
            }
        };
        let result = self.user_collection.update_one(filter, update, None).await;
        match result {
            Ok(result) => match result.modified_count {
                1 => Ok(result),
                _ => Err(Error::DeserializationError {
                    message: "Failed to add document".to_string(),
                }),
            },
            Err(e) => {
                log::error!("Failed to update profile field for account {}", id);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    /// Remove profile field
    pub async fn remove_profile_field(
        &self,
        id: &String,
        target: String,
        value: serde_json::Value,
        date: i64,
    ) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id)
            .ok()
            .expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let field_id = value["field_id"].as_str().unwrap();
        let target = format!("profile.{}", target);
        // pull object from target array where skill = name
        let update = doc! {
            "$pull": {
                target: {
                    "field_id": field_id
                }
            },
            "$set": {
                "profile.date_updated": date,
            }
        };
        let result = self.user_collection.update_one(filter, update, None).await;
        match result {
            Ok(result) => match result.modified_count {
                1 => Ok(result),
                _ => Err(Error::DeserializationError {
                    message: "Failed to add document".to_string(),
                }),
            },
            Err(e) => {
                log::error!("Failed to remove profile field for account {}", id);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    pub async fn add_document(
        &self,
        id: &str,
        document: DocumentInfo,
        date: i64,
    ) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id)
            .ok()
            .expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let update = doc! {
            "$push": {
                "documents": {
                    "title": document.title.to_owned(),
                    "prompt": document.prompt.to_owned(),
                    "content": document.content.to_owned(),
                    "rating": Some(document.rating),
                }
            },
            "$set": {
                "date_updated": date,
            }
        };
        let result = self.user_collection.update_one(filter, update, None).await;
        match result {
            Ok(result) => match result.modified_count {
                1 => Ok(result),
                _ => Err(Error::DeserializationError {
                    message: "Failed to add document".to_string(),
                }),
            },
            Err(e) => {
                log::error!("Failed to add document for account {}", id);
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }

    #[allow(dead_code)]
    pub async fn drop_database(&self) -> Result<(), Error> {
        let result = self.user_collection.drop(None).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to drop database");
                Err(Error::DeserializationError {
                    message: e.to_string(),
                })
            }
        }
    }
}
