use crate::models::Account;
use mongodb::{
    bson::{extjson::de::Error, oid::ObjectId, doc},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    Client, Collection,
};
use log;
use dotenv::dotenv;
use std::env;

pub struct AccountRepository {
    collection: Collection<Account>,
}

impl AccountRepository {

    /// Initialize the repository with a MongoDB connection
    pub async fn new() -> Self {
        dotenv().ok();
        let user = env::var("MONGO_USER").expect("MONGO_USER must be set");
        let psw = env::var("MONGO_PASSWORD").expect("MONGO_PASSWORD must be set");
        let host = env::var("MONGO_HOST").expect("MONGO_HOST must be set");

        let uri = format!("mongodb+srv://{}:{}@{}/?retryWrites=true&w=majority", user.as_str(), psw.as_str(), host.as_str());
        let client_options = mongodb::options::ClientOptions::parse(uri).await.ok().expect("Failed to parse client options");
        let client = Client::with_options(client_options).ok().expect("Failed to initialize client");
        log::info!("Connected to MongoDB at {}", host);

        let db = client.database("scrippt");
        let collection: Collection<Account> = db.collection("accounts");

        AccountRepository { collection }
    }

    /// Get a single account by id
    /// id must be a valid ObjectId
    pub async fn get_account(&self, id: &str) -> Result<Account, Error> {
        let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        log::info!("Filter: {:?}", filter);
        let account_detail = self.collection.find_one(filter, None).await.ok().expect("Failed to execute find");
        log::info!("Account: {:?}", account_detail);
        Ok(account_detail.unwrap())
    }

    /// Create a new account
    /// Account must have a name, email, and password
    pub async fn create_account(&self, acc: Account) -> Result<InsertOneResult, Error> {
        let new_doc = Account {
            id: None,
            name: acc.name,
            email: acc.email,
            password: acc.password,
        };
        let acc = self.collection.insert_one(new_doc, None).await.ok().expect("Failed to insert document");
        Ok(acc)
    }

    pub async fn update_account(&self, id: &str, acc: Account) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        log::info!("Filter: {:?}", filter);
        let new_doc = doc! {
            "$set": {
                "id": acc.id,
                "name": acc.name,
                "email": acc.email,
                "password": acc.password,
            }
        };
        let updated_doc = self.collection.update_one(filter, new_doc, None).await.ok().expect("Failed to update document");
        Ok(updated_doc)
    }

    pub async fn delete_account(&self, id: &str) -> Result<DeleteResult, Error> {
        let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        log::info!("Filter: {:?}", filter);
        let account_detail = self.collection.delete_one(filter, None).await.ok().expect("Failed to execute find");
        log::info!("Account: {:?}", account_detail);
        Ok(account_detail)
    }

}