use crate::auth::jwt::decode_jwt;
use actix_web::{dev, error::ErrorUnauthorized, Error, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use std::env;

/// Authorization service extractor
///
/// Requires:
///     Authorization header with Bearer token
pub struct AuthorizationService {
    pub id: String,
}

impl FromRequest for AuthorizationService {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let auth = req.headers().get("Authorization");
        if auth.is_none() {
            return err(ErrorUnauthorized("No Authorization header"));
        }

        // Extract JWT from header
        let split: Vec<&str> = auth.unwrap().to_str().unwrap().split_whitespace().collect();
        if split[0] != "Bearer" {
            return err(ErrorUnauthorized("Token is not a Bearer token"));
        }
        let token = split[1].trim();
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");

        // Decode JWT
        match decode_jwt(token.to_string(), &secret) {
            Ok(claims) => ok(AuthorizationService { id: claims.sub }),
            Err(_) => err(ErrorUnauthorized("Invalid token")),
        }
    }
}
