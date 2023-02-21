use crate::models::user::Claims;
use bson::Uuid;
use jsonwebtoken::{
    decode, decode_header, encode,
    errors::{Error, ErrorKind},
    Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct GoogleAuthClaims {
    iss: String,
    nbf: usize,
    aud: String,
    sub: String,
    hd: String,
    email: String,
    email_verified: String,
    azp: String,
    name: String,
    picture: String,
    given_name: String,
    family_name: String,
    iat: usize,
    exp: usize,
    jti: String,
    alg: String,
    kid: String,
    typ: String,
}

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

pub fn decode_jwt(token: String, secret: &str) -> Result<Claims, Error> {
    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(decoded.claims)
}

pub async fn jwk_request_helper() -> Result<serde_json::Value, Error> {
    let res = reqwest::get("https://www.googleapis.com/oauth2/v3/certs")
        .await
        .unwrap();
    // Get Cache-Control header
    let cache_control = res.headers().get("Cache-Control").unwrap();
    // Get max-age value from Cache-Control header
    let max_age = cache_control
        .to_str()
        .unwrap()
        .split(',')
        .find(|&s| s.starts_with("max-age="))
        .unwrap()
        .split('=')
        .nth(1)
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let jwk = res.text().await.unwrap();

    // Add max-age to JWKs
    let mut jwk: serde_json::Value = serde_json::from_str(&jwk)?;
    jwk["max_age"] = serde_json::Value::from(max_age);

    // Write JWKs to google_jwk.json
    fs::write("google_jwk.json", serde_json::to_string(&jwk).unwrap()).unwrap();
    Ok(jwk)
}

pub async fn decode_google_token_id(token: &str) -> Result<GoogleAuthClaims, Error> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap();

    // Retrieve JWKs from google_jwk.json if it exists and if the max_age JSON field is not greater than last modified.
    // Else retrieve JWKs from Google's JWK endpoint.
    let jwk = if Path::new("./google_jwk.json").exists() {
        let metadata = fs::metadata("google_jwk.json").unwrap();
        let modified = metadata.modified().unwrap();
        let jwk = fs::read_to_string("google_jwk.json")
            .unwrap()
            .parse::<serde_json::Value>()
            .unwrap();
        let max_age = jwk["max_age"].as_u64().unwrap();

        // If the max-age JSON field is not greater than last modified, use the cached JWKs.
        // Else retrieve JWKs from Google's JWK endpoint.
        if modified.elapsed().unwrap().as_secs() < max_age {
            jwk
        } else {
            let jwk = jwk_request_helper().await?;
            jwk
        }
    } else {
        let jwk = jwk_request_helper().await?;
        jwk
    };

    // Find the key that corresponds to the `kid` in the token's header.
    let header = decode_header(&token)?;
    let kid = header.kid.ok_or_else(|| ErrorKind::InvalidKeyFormat)?;

    // Find the JWK key that corresponds to the `kid` in the token's header.
    let jwk_key = jwk["keys"]
        .as_array()
        .ok_or_else(|| ErrorKind::InvalidKeyFormat)?
        .iter()
        .find(|key| key["kid"] == kid)
        .ok_or_else(|| ErrorKind::InvalidKeyFormat)?;

    // Construct the decoding key from the JWK key.
    let decoding_key = DecodingKey::from_rsa_components(
        &jwk_key["n"].as_str().unwrap(),
        &jwk_key["e"].as_str().unwrap(),
    )?;

    // Decode and verify the token.
    let claims =
        decode::<GoogleAuthClaims>(&token, &decoding_key, &Validation::new(Algorithm::RS256))?;

    // Verify the client ID, issuer, and expiration.
    if claims.claims.aud != client_id {
        return Err(ErrorKind::InvalidAudience.into());
    }

    if claims.claims.iss != "accounts.google.com"
        || claims.claims.iss != "https://accounts.google.com"
    {
        return Err(ErrorKind::InvalidIssuer.into());
    }

    if claims.claims.exp < chrono::Utc::now().timestamp() as usize {
        return Err(ErrorKind::ExpiredSignature.into());
    }

    Ok(claims.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_decode_google_token_id() {
        // set GOOGLE_CLIENT_ID env var
        std::env::set_var(
            "GOOGLE_CLIENT_ID",
            "82324295624-32uqo7r4j24etafpr2t0ddqt5b0etmj8.apps.googleusercontent.com",
        );

        let test_id = "eyJhbGciOiJSUzI1NiIsImtpZCI6IjU5NjJlN2EwNTljN2Y1YzBjMGQ1NmNiYWQ1MWZlNjRjZWVjYTY3YzYiLCJ0eXAiOiJKV1QifQ.eyJpc3MiOiJodHRwczovL2FjY291bnRzLmdvb2dsZS5jb20iLCJuYmYiOjE2NzY5NjMwNjgsImF1ZCI6IjgyMzI0Mjk1NjI0LTMydXFvN3I0ajI0ZXRhZnByMnQwZGRxdDViMGV0bWo4LmFwcHMuZ29vZ2xldXNlcmNvbnRlbnQuY29tIiwic3ViIjoiMTAwNTk5NzkxMDI4ODg5MzE5NDYxIiwiaGQiOiJ1bWljaC5lZHUiLCJlbWFpbCI6InNhbnRpYWdtQHVtaWNoLmVkdSIsImVtYWlsX3ZlcmlmaWVkIjp0cnVlLCJhenAiOiI4MjMyNDI5NTYyNC0zMnVxbzdyNGoyNGV0YWZwcjJ0MGRkcXQ1YjBldG1qOC5hcHBzLmdvb2dsZXVzZXJjb250ZW50LmNvbSIsIm5hbWUiOiJTYW50aWFnbyBNZWRpbmEgUm9sb25nIiwicGljdHVyZSI6Imh0dHBzOi8vbGgzLmdvb2dsZXVzZXJjb250ZW50LmNvbS9hL0FFZEZUcDRsS0ZfT3FNejEwc3pPUVpfcWp3TlFjRkF5S09helhuWFQxaFNmPXM5Ni1jIiwiZ2l2ZW5fbmFtZSI6IlNhbnRpYWdvIiwiZmFtaWx5X25hbWUiOiJNZWRpbmEgUm9sb25nIiwiaWF0IjoxNjc2OTYzMzY4LCJleHAiOjE2NzY5NjY5NjgsImp0aSI6IjNjYmUxZjQ2ZmE0MzQwZTZkMGI5NWEzYTdhMDMxNWI2YzMzMGJhOTgifQ.FQkTdEKHNmuipbzZDzeXfki0bjsZzLrJvR0l6R5sAC8u-GAVEVnz2OJJu7evjyWv1DgvdCO_P8fyGT9nm3pWXw7PbVeYbbd-QzlFMY8lU_k9VRNWGzQlXxMo9jWNzCZSGOoTnoKyjDdI9O7M42aVRHBybBFpfBhDEXwpXRwBK4kybTJswB-BZ9ILzsKVyB5LzMKGcl9nuW8UDB0Cfq3A2u6WPOnuEAn8ts6Z4vlTkadN-OxVWFCeF4eHqniu2i6hZkM_6YE6CzX0Kdkx11h3LWSw5gNOl6QVFM5nV9Be1eLjhaaIQEmNCo81VoGTyNPdexYur35q5wJJweffO4LN0w";

        let claims = decode_google_token_id(&test_id).await.unwrap();
        assert_eq!(claims.email, "santiagm@umich.edu");
    }
}
