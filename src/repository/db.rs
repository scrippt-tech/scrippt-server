use std::env;
use log;
use mongodb::{
    Client, Collection,
    bson::{extjson::de::Error, doc},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    bson::oid::ObjectId,
};

use crate::models::user::{User, UserUpdate};
use crate::models::profile::ProfileInfo;

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
                                                .await
                                                .ok()
                                                .expect("Failed to execute find");
            match account_detail {
                Some(account) => Ok(account),
                None => Err(Error::DeserializationError { message: "Account not found".to_string() })
            }
        }
    
        pub async fn get_account_by_email(&self, email: &str) -> Result<User, Error> {
            let filter = doc! {"email": email};
            let account_detail = self.user_collection
                                                .find_one(filter, None)
                                                .await
                                                .ok()
                                                .expect("Failed to execute find");
            match account_detail {
                Some(account) => Ok(account),
                None => Err(Error::DeserializationError { message: "Account not found".to_string() })
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
                                            .await
                                            .ok()
                                            .expect("Failed to insert document");
            Ok(user)
        }
    
        pub async fn update_account(&self, id: &str, user: &UserUpdate) -> Result<UpdateResult, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
            let filter = doc! {"_id": obj_id};
            let new_doc = doc! {
                "$set": {
                    "name": user.name.to_owned(),
                    "email": user.email.to_owned(),
                    "password": user.password.to_owned(),
                    "date_updated": user.date_updated,
                }
            };
            let updated_doc = self.user_collection
                                                .update_one(filter, new_doc, None)
                                                .await
                                                .ok()
                                                .expect("Failed to update document");
            Ok(updated_doc)
        }
    
        pub async fn delete_account(&self, id: &str) -> Result<DeleteResult, Error> {
            let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
            let filter = doc! {"_id": obj_id};
            let account_detail = self.user_collection
                                                .delete_one(filter, None)
                                                .await
                                                .ok()
                                                .expect("Failed to execute find");
            Ok(account_detail)
        }

        /// Update a profile
        /// Requires:
        ///     id must be a valid ObjectId
        ///     profile must be a valid profile
        ///     date must be a valid timestamp
        pub async fn update_profile(&self, account_id: &str, profile: &ProfileInfo, date: i64) -> Result<UpdateResult, Error> {
            let filter = doc! {"_id": account_id};
            let update = doc! {
                "$set": {
                    "profile": profile.to_owned(),
                    "date_updated": date,
                }
            };
            let result = self.user_collection
                                        .update_one(filter, update, None)
                                        .await
                                        .ok()
                                        .expect("Failed to update document");
            Ok(result)
        }

}