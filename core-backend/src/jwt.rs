//! Emisión y validación de JWT de acceso.

use crate::app_config::JwtConfig;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Header, Validation};
use serde::{Deserialize, Serialize};

const TOKEN_TYPE: &str = "access";

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: i32,
    pub username: String,
    pub is_admin: bool,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

pub fn create_access_token(
    jwt: &JwtConfig,
    user_id: i32,
    username: &str,
    is_admin: bool,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = AccessClaims {
        sub: user_id,
        username: username.to_string(),
        is_admin,
        iat: now.timestamp(),
        exp: (now + Duration::seconds(jwt.access_ttl_secs)).timestamp(),
        token_type: TOKEN_TYPE.to_string(),
    };

    encode(&Header::default(), &claims, jwt.encoding_key())
}

pub fn validate_access_token(jwt: &JwtConfig, token: &str) -> Result<AccessClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<AccessClaims>(token, jwt.decoding_key(), &validation)?;
    if token_data.claims.token_type != TOKEN_TYPE {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }
    Ok(token_data.claims)
}
