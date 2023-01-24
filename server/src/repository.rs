use mongodb::{
    bson::{extjson::de::Error, oid::ObjectId, doc},
    results::InsertOneResult,
    Client, Collection,
};
use log;
use crate::models::Account;

pub struct AccountRepository {
    collection: Collection<Account>,
}

impl AccountRepository {

    /// Initialize the repository with a MongoDB connection
    pub async fn new() -> Self {
        let client = Client::with_uri_str("mongodb://localhost:27017").await.ok().expect("Failed to initialize client");
        log::info!("Connected to MongoDB at localhost:27017");
        let db = client.database("scrippt");
        let collection: Collection<Account> = db.collection("accounts");

        AccountRepository { collection }
    }

    /// Get a single account by id
    /// id must be a valid ObjectId
    pub async fn get_account(&self, id: &str) -> Result<Account, Error> {
        let obj_id = ObjectId::parse_str(id).ok().expect("Failed to parse object id");
        let filter = doc! {"_id": obj_id};
        let account_detail = self.collection.find_one(filter, None).await.ok().expect("Failed to execute find");
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
}