//! Emisión y validación de JWT de acceso.
//!
//! Los tokens son HS256 firmados con `JWT_SECRET`. Solo se aceptan tokens con
//! `token_type: "access"`; los refresh tokens nunca viajan como JWT.

use crate::app_config::JwtConfig;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Header, Validation};
use serde::{Deserialize, Serialize};

const TOKEN_TYPE: &str = "access";

/// Claims del access token JWT.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    /// ID del usuario (`sub`).
    pub sub: i32,
    /// Nombre de usuario.
    pub username: String,
    /// Rol administrador embebido en el token.
    pub is_admin: bool,
    /// Expiración Unix (segundos).
    pub exp: i64,
    /// Emitido en Unix (segundos).
    pub iat: i64,
    /// Debe ser `"access"` para distinguir de otros tipos de token.
    pub token_type: String,
}

/// Crea un access token firmado con la configuración JWT actual.
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

/// Decodifica y valida un access token (firma, expiración y `token_type`).
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
