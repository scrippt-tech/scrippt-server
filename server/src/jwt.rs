use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub email: String,
    pub exp: usize,
}

pub fn encode_jwt(id: String, email: String, secret: String) -> String {
    let my_claims = Claims {
        id,
        email,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ).unwrap();

    token
}

pub fn decode_jwt(token: String, secret: String) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(decoded.claims)
}