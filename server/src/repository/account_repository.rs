use crate::models::account::Account;
use crate::repository::db::DatabaseRepository;
use mongodb::{
    bson::{extjson::de::Error, doc},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    bson::oid::ObjectId,
};


impl DatabaseRepository {

    pub async fn get_account(&self, id: &str) -> Result<Account, Error> {
        let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let account_detail = self.account_collection.find_one(filter, None).await.ok().expect("Failed to execute find");
        Ok(account_detail.unwrap())
    }

    pub async fn get_account_by_email(&self, email: &str) -> Result<Account, Error> {
        let filter = doc! {"email": email};
        let account_detail = self.account_collection.find_one(filter, None).await.ok().expect("Failed to execute find");
        match account_detail {
            Some(account) => Ok(account),
            None => Err(Error::DeserializationError { message: (
                format!("Account with email {} not found", email)
            )})
        }
    }

    pub async fn create_account(&self, acc: Account) -> Result<InsertOneResult, Error> {
        let new_doc = Account {
            id: None,
            name: acc.name,
            email: acc.email,
            password: acc.password,
            date_created: acc.date_created,
            date_updated: acc.date_updated
        };
        let acc = self.account_collection.insert_one(new_doc, None).await.ok().expect("Failed to insert document");
        Ok(acc)
    }

    pub async fn update_account(&self, id: &str, acc: Account) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let new_doc = doc! {
            "$set": {
                "id": acc.id,
                "name": acc.name,
                "email": acc.email,
                "password": acc.password,
                "date_created": acc.date_created,
                "date_updated": acc.date_updated
            }
        };
        let updated_doc = self.account_collection.update_one(filter, new_doc, None).await.ok().expect("Failed to update document");
        Ok(updated_doc)
    }

    pub async fn delete_account(&self, id: &str) -> Result<DeleteResult, Error> {
        let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let account_detail = self.account_collection.delete_one(filter, None).await.ok().expect("Failed to execute find");
        Ok(account_detail)
    }

}