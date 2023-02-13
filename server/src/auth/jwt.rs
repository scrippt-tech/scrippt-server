use crate::models::user::Claims;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

pub fn encode_jwt(id: String, email: String, secret: &str) -> String {
    let my_claims = Claims {
        sub: id,
        email,
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

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    // call a function to set up the environment
    fn setup() {
        dotenv().ok();
    }

    #[test]
    #[ignore = "This test requires a JWT_SECRET to be set"]
    fn test_encode_jwt() {
        setup();
        let secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let my_claims = Claims {
            sub: "1234".to_string(),
            email: "sant@gmail".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };

        let correct_token = encode(
            &Header::default(),
            &my_claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        let token = encode_jwt("1234".to_string(), "sant@gmail".to_string(), &secret);

        assert_eq!(token, correct_token);
    }

    #[test]
    #[ignore = "This test requires a JWT_SECRET to be set"]
    fn test_decode_jwt() {
        setup();
        let secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let token = encode_jwt("1234".to_string(), "sant@gmail".to_string(), &secret);
        let decoded = decode_jwt(token, &secret).unwrap();
        assert_eq!(decoded.sub, "1234");
        assert_eq!(decoded.email, "sant@gmail");
    }
}
