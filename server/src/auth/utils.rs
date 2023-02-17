use argon2;
use email_address_parser::EmailAddress;
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

pub fn validate_signup(email: &str, password: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("Email cannot be empty".to_string());
    }
    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }
    if !EmailAddress::is_valid(email, None) {
        return Err("Invalid email".to_string());
    }
    Ok(())
}
