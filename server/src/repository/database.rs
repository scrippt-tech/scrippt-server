use std::env;
use log;
use serde_json;
use mongodb::{
    Client, Collection,
    bson::{extjson::de::Error, doc},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    bson::oid::ObjectId,
};
use bson::to_bson;

use crate::models::{user::{User, UserUpdate}};
use crate::models::profile::Profile;
use crate::models::document::DocumentInfo;

pub struct DatabaseRepository {
    pub user_collection: Collection<User>,
}

impl DatabaseRepository {
        /// Initialize the repository with a MongoDB connection
        pub async fn new() -> Self {
            let user = env::var("MONGO_USER").expect("MONGO_USER must be set");
            let psw = env::var("MONGO_PASSWORD").expect("MONGO_PASSWORD must be set");
            let host = env::var("MONGO_HOST").expect("MONGO_HOST must be set");
    
            let uri = format!("mongodb+srv://{}:{}@{}/?retryWrites=true&w=majority", user.as_str(), psw.as_str(), host.as_str());
            let client_options = mongodb::options::ClientOptions::parse(uri)
                                                                                    .await
                                                                                    .ok()
                                                                                    .expect("Failed to parse client options");
            let client = Client::with_options(client_options).ok().expect("Failed to initialize client");
            
            log::info!("Connected to MongoDB at {}", host);
    
            let db = client.database("scrippt");
            let user_collection: Collection<User> = db.collection("users");

            DatabaseRepository { user_collection }
        }

        pub async fn get_account(&self, id: &str) -> Result<User, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
            let filter = doc! {"_id": obj_id};
            let account_detail = self.user_collection
                                                .find_one(filter, None)
                                                .await;
            match account_detail {
                Ok(Some(account)) => Ok(account),
                Ok(None) => Err(Error::DeserializationError { message: "Account not found".to_string() }),
                Err(e) => {
                    log::error!("Failed to get account {}", id);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }
    
        pub async fn get_account_by_email(&self, email: &str) -> Result<User, Error> {
            let filter = doc! {"email": email};
            let account_detail = self.user_collection
                                                .find_one(filter, None)
                                                .await;
            match account_detail {
                Ok(Some(account)) => Ok(account),
                Ok(None) => Err(Error::DeserializationError { message: "Account not found".to_string() }),
                Err(e) => {
                    log::error!("Failed to get account by email {}", email);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }
    
        pub async fn create_account(&self, user: User) -> Result<InsertOneResult, Error> {
            let new_doc = User {
                id: None,
                name: user.name,
                email: user.email,
                password: user.password,
                profile: user.profile,
                documents: user.documents,
                date_created: user.date_created,
                date_updated: user.date_updated
            };
            let user = self.user_collection
                                            .insert_one(new_doc, None)
                                            .await;
            match user {
                Ok(result) => Ok(result),
                Err(e) => {
                    log::error!("Failed to create account {}", e);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }
    
        pub async fn update_account(&self, id: &str, user: &UserUpdate) -> Result<UpdateResult, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
            let filter = doc! {"_id": obj_id};
            let new_doc = doc! {
                "$set": {
                    "name": user.name.to_owned(),
                    "email": user.email.to_owned(),
                    "date_updated": user.date_updated,
                }
            };
            let updated_doc = self.user_collection
                                                .update_one(filter, new_doc, None)
                                                .await;
            match updated_doc {
                Ok(result) => {
                    match result.modified_count {
                        1 => Ok(result),
                        _ => Err(Error::DeserializationError { message: "Failed to update document".to_string() })
                    }
                },
                Err(e) => {
                    log::error!("Failed to update account {}", id);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }
    
        pub async fn delete_account(&self, id: &str) -> Result<DeleteResult, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
            let filter = doc! {"_id": obj_id};
            let account_detail = self.user_collection
                                                .delete_one(filter, None)
                                                .await;
            match account_detail {
                Ok(result) => {
                    match result.deleted_count {
                        1 => Ok(result),
                        _ => Err(Error::DeserializationError { message: "Failed to delete document".to_string() })
                    }
                },
                Err(e) => {
                    log::error!("Failed to delete account {}", id);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }

        /// Create a profile
        /// Requires:
        ///    id must be a valid ObjectId
        ///    profile must be a valid profile
        ///    date must be a valid timestamp
        ///
        pub async fn create_profile(&self, id: &str, profile: Profile, date: i64) -> Result<UpdateResult, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
            let filter = doc! {"_id": obj_id};

            let update = doc! {
                "$set": {
                    "profile.education": to_bson(&profile.education).unwrap(),
                    "profile.experience": to_bson(&profile.experience).unwrap(),
                    "profile.skills": to_bson(&profile.skills).unwrap(),
                    "date_updated": date,
                }
            };

            let result = self.user_collection
                                        .update_one(filter, update, None)
                                        .await;
            match result {
                Ok(result) => {
                    match result.modified_count {
                        1 => Ok(result),
                        _ => Err(Error::DeserializationError { message: "Failed to add document".to_string() })
                    }
                },
                Err(e) => {
                    log::error!("Failed to create profile for account {}", id);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }

        /// Add profile field
        /// 
        /// ## Arguments:
        /// - id: String
        /// - target: String
        /// - value: serde_json::Value
        /// - date: i64
        /// 
        /// ## Returns:
        /// - UpdateResult
        /// 
        pub async fn add_profile_field(&self, id: &String, target: String, value: serde_json::Value, date: i64) -> Result<UpdateResult, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
            let filter = doc! {"_id": obj_id};
            let target = format!("profile.{}", target);
            let update = doc! {
                "$push": {
                    target: to_bson(&value).unwrap()
                },
                "$set": {
                    "profile.date_updated": date,
                }
            };
            let result = self.user_collection
                                        .update_one(filter, update, None)
                                        .await;
            match result {
                Ok(result) => {
                    match result.modified_count {
                        1 => Ok(result),
                        _ => Err(Error::DeserializationError { message: "Failed to add document".to_string() })
                    }
                },
                Err(e) => {
                    log::error!("Failed to add profile field for account {}", id);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }

        /// Update profile field
        /// 
        /// ## Arguments:
        /// - id: ObjectId of the user
        /// - target: the field to be updated
        /// - value: the new value for the field
        /// - date: the timestamp of the update
        /// 
        /// ## Returns:
        /// - UpdateResult
        /// 
        pub async fn update_profile_field(&self, id: &String, target: String, value: serde_json::Value, date: i64) -> Result<UpdateResult, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
            let filter = doc! {"_id": obj_id};
            let target = format!("profile.{}", target);
            let update = doc! {
                "$set": {
                    target: to_bson(&value).unwrap(),
                    "profile.date_updated": date,
                }
            };
            let result = self.user_collection
                                        .update_one(filter, update, None)
                                        .await;
            match result {
                Ok(result) => {
                    match result.modified_count {
                        1 => Ok(result),
                        _ => Err(Error::DeserializationError { message: "Failed to add document".to_string() })
                    }
                },
                Err(e) => {
                    log::error!("Failed to update profile field for account {}", id);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }

        /// Remove profile field
        /// 
        /// ## Arguments:
        /// - id: ObjectId of the user
        /// - target: the field to be removed
        /// - date: the timestamp of the update
        /// 
        /// ## Returns:
        /// - UpdateResult
        /// 
        pub async fn remove_profile_field(&self, id: &String, target: String, value: serde_json::Value, date: i64) -> Result<UpdateResult, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
            let name = value["skill"].as_str().unwrap();
            let target = format!("profile.{}", target);
            let filter = doc! {"_id": obj_id};
            // pull object from target array where skill = name
            let update = doc! {
                "$pull": {
                    target: {
                        "skill": name
                    }
                },
                "$set": {
                    "profile.date_updated": date,
                }
            };
            let result = self.user_collection
                                        .update_one(filter, update, None)
                                        .await;
            match result {
                Ok(result) => {
                    match result.modified_count {
                        1 => Ok(result),
                        _ => Err(Error::DeserializationError { message: "Failed to add document".to_string() })
                    }
                },
                Err(e) => {
                    log::error!("Failed to remove profile field for account {}", id);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }


        pub async fn add_document(&self, id: &str, document: DocumentInfo, date: i64) -> Result<UpdateResult, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
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
            let result = self.user_collection
                                        .update_one(filter, update, None)
                                        .await;
            match result {
                Ok(result) => {
                    match result.modified_count {
                        1 => Ok(result),
                        _ => Err(Error::DeserializationError { message: "Failed to add document".to_string() })
                    }
                },
                Err(e) => {
                    log::error!("Failed to add document for account {}", id);
                    Err(Error::DeserializationError { message: e.to_string() })
                }
            }
        }

}