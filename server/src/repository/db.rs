use crate::models::{profile::Profile, account:: Account};
use mongodb::{Client, Collection};
use std::env;
use log;

pub struct DatabaseRepository {
    pub profile_collection: Collection<Profile>,
    pub account_collection: Collection<Account>,
}

impl DatabaseRepository {
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
            let profile_collection: Collection<Profile> = db.collection("profiles");
            let account_collection: Collection<Account> = db.collection("accounts");

            DatabaseRepository { profile_collection, account_collection }
        }
}