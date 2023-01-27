use crate::models::profile::Profile;
use mongodb::{
    bson::{extjson::de::Error, doc},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    Client, Collection,
};
use std::env;
use log;

pub struct ProfileRepository {
    collection: Collection<Profile>,
}

impl ProfileRepository {

    /// Initialize the repository with a MongoDB connection
    pub async fn new() -> Self {
        let user = env::var("MONGO_USER").expect("MONGO_USER must be set");
        let psw = env::var("MONGO_PASSWORD").expect("MONGO_PASSWORD must be set");
        let host = env::var("MONGO_HOST").expect("MONGO_HOST must be set");

        let uri = format!("mongodb+srv://{}:{}@{}/?retryWrites=true&w=majority", user.as_str(), psw.as_str(), host.as_str());
        let client_options = mongodb::options::ClientOptions::parse(uri).await.ok().expect("Failed to parse client options");
        let client = Client::with_options(client_options).ok().expect("Failed to initialize client");
        log::info!("Connected to MongoDB at {}", host);

        let db = client.database("scrippt");
        let collection: Collection<Profile> = db.collection("profiles");

        ProfileRepository { collection }
    }

    /// Get a single profile by id
    /// id must be a valid ObjectId
    pub async fn get_profile(&self, account_id: &str) -> Result<Profile, Error> {
        let filter = doc! {"account_id": account_id};
        let profile_detail = self.collection.find_one(filter, None).await.ok().expect("Failed to execute find");
        Ok(profile_detail.unwrap())
    }

    /// Create a new profile
    pub async fn create_profile(&self, profile: Profile) -> Result<InsertOneResult, Error> {
        let new_doc = Profile {
            account_id: profile.account_id,
            profile: profile.profile,
            date_updated: profile.date_updated,
        };
        let result = self.collection.insert_one(new_doc, None).await.ok().expect("Failed to insert document");
        Ok(result)
    }

    /// Update a profile
    /// id must be a valid ObjectId
    /// profile must be a valid profile
    pub async fn update_profile(&self, account_id: &str, profile: Profile) -> Result<UpdateResult, Error> {
        let filter = doc! {"account_id": account_id};
        let update = doc! {
            "$set": {
                "profile": profile.profile,
                "date_updated": profile.date_updated,
            }
        };
        let result = self.collection.update_one(filter, update, None).await.ok().expect("Failed to update document");
        Ok(result)
    }

    /// Delete a profile
    /// id must be a valid ObjectId
    /// profile must be a valid profile
    pub async fn delete_profile(&self, account_id: &str) -> Result<DeleteResult, Error> {
        let filter = doc! {"account_id": account_id};
        let result = self.collection.delete_one(filter, None).await.ok().expect("Failed to delete document");
        Ok(result)
    }
}