use argon2;
use email_address_parser::EmailAddress;
use rand::Rng;

// Password hashing

/// Generate a hash from a password
pub fn generate_hash(password: &str) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = argon2::Config::default();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &config).unwrap();
    hash
}

/// Verify a password against a hash
pub fn verify_hash(password: &str, hash: &str) -> bool {
    argon2::verify_encoded(hash, password.as_bytes()).unwrap()
}

// Information validation

/// Validate a user's email and password
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

// OTP utils

/// Generate a random 6 digit OTP using the current time as a seed
pub fn generate_otp_code() -> String {
    let mut rng = rand::thread_rng();
    let otp: u32 = rng.gen_range(100000..999999);
    otp.to_string()
}

/// Get expiration time for an OTP code given minutes to expire
pub fn get_expiration_time(minutes: usize) -> usize {
    let now = chrono::Utc::now();
    let expiration_time = now + chrono::Duration::minutes(minutes as i64);
    expiration_time.timestamp() as usize
}
