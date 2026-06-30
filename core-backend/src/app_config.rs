//! Configuración JWT y estado compartido de la aplicación.

use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

/// Parámetros de firma y caducidad de tokens.
#[derive(Clone)]
pub struct JwtConfig {
    encoding: EncodingKey,
    decoding: DecodingKey,
    pub access_ttl_secs: i64,
    pub refresh_ttl_secs: i64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            eprintln!("ADVERTENCIA: JWT_SECRET no definido; usando valor solo para desarrollo.");
            "dev-only-change-me".to_string()
        });

        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            access_ttl_secs: env_i64("JWT_ACCESS_TTL_SECS", 3600),
            refresh_ttl_secs: env_i64("JWT_REFRESH_TTL_SECS", 604_800),
        }
    }

    pub fn encoding_key(&self) -> &EncodingKey {
        &self.encoding
    }

    pub fn decoding_key(&self) -> &DecodingKey {
        &self.decoding
    }
}

/// Pool de BD + configuración JWT.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt: JwtConfig,
}

fn env_i64(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
