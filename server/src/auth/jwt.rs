use crate::models::user::Claims;
use bson::Uuid;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

pub fn encode_jwt(iss: String, sub: String, aud: String, secret: &str) -> String {
    let my_claims = Claims {
        iss,
        sub,
        aud,
        iat: chrono::Utc::now().timestamp() as usize,
        nbf: chrono::Utc::now().timestamp() as usize,
        jti: Uuid::new().to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();

    token
}

pub fn decode_jwt(token: String, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(decoded.claims)
}
