use crate::models::user::Claims;
use bson::Uuid;
use jsonwebtoken::{
    decode, decode_header, encode,
    errors::{Error, ErrorKind},
    jwk::{Jwk, JwkSet},
    Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use reqwest;
use serde::{Deserialize, Serialize};
// #[cfg(feature = "json")]
use serde_json;
use std::{fs, path::Path};

/// Encode a JWT with the given claims.
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

/// Decode a JWT with the given claims.
pub fn decode_jwt(token: String, secret: &str) -> Result<Claims, Error> {
    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(decoded.claims)
}

/// A Google ID token claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleAuthClaims {
    pub iss: String,
    pub nbf: usize,
    pub aud: String,
    pub sub: String,
    pub hd: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub azp: String,
    pub name: String,
    pub picture: String,
    pub given_name: String,
    pub family_name: String,
    pub iat: usize,
    pub exp: usize,
    pub jti: String,
}

/// A JWK set from Google.
/// Includes a max_age field that is not part of the JWK set standard.
/// This field is used to determine if the JWK set should be updated.
#[derive(Debug, Serialize, Deserialize)]
struct GoogleJwkSet {
    keys: Vec<Jwk>,
    max_age: u64,
}

impl std::str::FromStr for GoogleJwkSet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let jwk_set: GoogleJwkSet = serde_json::from_str(s)?;
        Ok(jwk_set)
    }
}

/// Decode a Google ID token.
///
/// # Usage
/// ```rust
/// use crate::auth::jwt::decode_google_token_id;
///
/// let claims = decode_google_token_id("token").await;
/// ```
pub async fn decode_google_token_id(token: &str) -> Result<GoogleAuthClaims, Error> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set");
    let key_path: &str = &std::env::var("GOOGLE_JWK_PATH").expect("GOOGLE_JWK_PATH must be set");

    // Retrieve JWKs from google_jwk.json if it exists and if the max_age JSON field is not greater than last modified.
    // Else retrieve JWKs from Google's JWK endpoint.
    let jwk_set = if Path::new(key_path).exists() {
        log::debug!(".jwk file found; retrieving JWKs from .jwk file.");
        let metadata = fs::metadata(key_path).unwrap();
        let modified = metadata.modified().unwrap();
        let jwk_set = fs::read_to_string(key_path)
            .unwrap()
            .parse::<GoogleJwkSet>()
            .unwrap();
        let max_age = jwk_set.max_age;

        // If the max-age JSON field is not greater than last modified, use the cached JWKs.
        // Else retrieve JWKs from Google's JWK endpoint.
        if modified.elapsed().unwrap().as_secs() < max_age {
            jwk_set
        } else {
            let jwk_set = get_latest_keys(key_path).await?;
            jwk_set
        }
    } else {
        let jwk_set = get_latest_keys(key_path).await?;
        jwk_set
    };

    // Find the key that corresponds to the `kid` in the token's header.
    let header = decode_header(&token)?;
    let kid = header.kid.ok_or_else(|| ErrorKind::InvalidKeyFormat)?;

    // Find the JWK key that corresponds to the `kid` in the token's header.
    let jwk_set = JwkSet { keys: jwk_set.keys };
    let jwk = jwk_set
        .find(&kid)
        .ok_or_else(|| ErrorKind::InvalidKeyFormat)?;

    // Construct the decoding key from the JWK key.
    let decoding_key = DecodingKey::from_jwk(jwk)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[client_id.as_str()]);
    validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);

    // Decode and verify the token.
    let claims = decode::<GoogleAuthClaims>(&token, &decoding_key, &validation);

    match claims {
        Ok(claims) => Ok(claims.claims),
        Err(err) => {
            log::error!("Error decoding token: {:?}", err);
            return Err(err);
        }
    }
}

/// Retrieve the latest JWKs from Google's JWK endpoint.
/// Cache the JWKs in a JSON file with a max-age field.
/// Return the JWKs.
async fn get_latest_keys(file_path: &str) -> Result<GoogleJwkSet, Error> {
    log::debug!("Retrieving latest JWKs from Google's JWK endpoint.");
    let res = reqwest::get("https://www.googleapis.com/oauth2/v3/certs")
        .await
        .unwrap();
    // Get Cache-Control header
    let cache_control = res.headers().get("cache-control").unwrap();
    // Get max-age value from Cache-Control header
    let max_age = cache_control
        .to_str()
        .unwrap()
        .split(',')
        .map(|s| s.trim())
        .find(|&s| s.starts_with("max-age="))
        .unwrap()
        .split('=')
        .nth(1)
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let jwk = res.json::<JwkSet>().await.unwrap();
    let jwk = GoogleJwkSet {
        max_age,
        keys: jwk.keys,
    };

    // Write JWKs to google_jwk.json
    fs::write(&file_path, serde_json::to_string(&jwk).unwrap()).unwrap_or_else(|err| {
        log::error!("Error writing JWKs to file {:?}: {:?}", file_path, err);
        panic!();
    });

    Ok(jwk)
}
