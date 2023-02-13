use crate::auth::jwt::decode_jwt;
use actix_web::{dev, error::ErrorUnauthorized, Error, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use std::env;

/// Authorization service extractor
/// Requires:
///     Authorization header with Bearer token
///     id in url
pub struct AuthorizationService;

impl FromRequest for AuthorizationService {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let id = req.match_info().get("id").unwrap();
        let auth = req.headers().get("Authorization");
        match auth {
            Some(_) => {
                let split: Vec<&str> = auth.unwrap().to_str().unwrap().split("Bearer").collect();
                let token = split[1].trim();
                let key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
                match decode_jwt(token.to_string(), &key) {
                    Ok(claims) => match claims.sub == id {
                        true => ok(AuthorizationService),
                        false => err(ErrorUnauthorized("invalid token")),
                    },
                    Err(_e) => err(ErrorUnauthorized("invalid token")),
                }
            }
            None => err(ErrorUnauthorized("unauthorized")),
        }
    }
}
