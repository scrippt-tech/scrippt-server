use crate::models::profile::Profile;
use crate::repository::db::DatabaseRepository;
use mongodb::{
    bson::{extjson::de::Error, doc},
    results::{InsertOneResult, UpdateResult, DeleteResult},
};

impl DatabaseRepository {

    /// Get a single profile by id
    /// id must be a valid ObjectId
    pub async fn get_profile(&self, account_id: &str) -> Result<Profile, Error> {
        let filter = doc! {"account_id": account_id};
        let profile_detail = self.profile_collection.find_one(filter, None).await.ok().expect("Failed to execute find");
        Ok(profile_detail.unwrap())
    }

    /// Create a new profile
    pub async fn create_profile(&self, profile: Profile) -> Result<InsertOneResult, Error> {
        let new_doc = Profile {
            account_id: profile.account_id,
            profile: profile.profile,
            date_updated: profile.date_updated,
        };
        let result = self.profile_collection.insert_one(new_doc, None).await.ok().expect("Failed to insert document");
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
        let result = self.profile_collection.update_one(filter, update, None).await.ok().expect("Failed to update document");
        Ok(result)
    }

    /// Delete a profile
    /// id must be a valid ObjectId
    /// profile must be a valid profile
    pub async fn delete_profile(&self, account_id: &str) -> Result<DeleteResult, Error> {
        let filter = doc! {"account_id": account_id};
        let result = self.profile_collection.delete_one(filter, None).await.ok().expect("Failed to delete document");
        Ok(result)
    }
}