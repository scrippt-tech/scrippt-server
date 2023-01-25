use argon2;
use rand::Rng;

pub fn generate_hash(password: &str) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = argon2::Config::default();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &config).unwrap();
    hash
}

pub fn verify_hash(password: &str, hash: &str) -> bool {
    argon2::verify_encoded(hash, password.as_bytes()).unwrap()
}