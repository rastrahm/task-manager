//! Configuración JWT y estado compartido de la aplicación.
//!
//! [`AppState`] se inyecta en todos los handlers Axum mediante `State<AppState>`.
//! [`JwtConfig`] centraliza claves de firma y tiempos de vida de los tokens.

use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

/// Parámetros de firma y caducidad de tokens JWT.
#[derive(Clone)]
pub struct JwtConfig {
    encoding: EncodingKey,
    decoding: DecodingKey,
    /// Segundos de validez del access token (`exp` en el JWT).
    pub access_ttl_secs: i64,
    /// Segundos de validez del refresh token almacenado en BD.
    pub refresh_ttl_secs: i64,
}

impl JwtConfig {
    /// Carga la configuración desde variables de entorno.
    ///
    /// | Variable | Default |
    /// |----------|---------|
    /// | `JWT_SECRET` | `dev-only-change-me` (con advertencia en stderr) |
    /// | `JWT_ACCESS_TTL_SECS` | `3600` |
    /// | `JWT_REFRESH_TTL_SECS` | `604800` |
    pub fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            eprintln!("ADVERTENCIA: JWT_SECRET no definido; usando valor solo para desarrollo.");
            "dev-only-change-me".to_string()
        });
        Self::from_secret(&secret)
    }

    /// Configuración fija para tests (unitarios e integración).
    #[cfg(any(test, feature = "test-utils"))]
    pub fn for_tests() -> Self {
        Self::from_secret("test-jwt-secret-only-for-tests")
    }

    fn from_secret(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            access_ttl_secs: env_i64("JWT_ACCESS_TTL_SECS", 3600),
            refresh_ttl_secs: env_i64("JWT_REFRESH_TTL_SECS", 604_800),
        }
    }

    /// Clave para firmar access tokens.
    pub fn encoding_key(&self) -> &EncodingKey {
        &self.encoding
    }

    /// Clave para verificar access tokens entrantes.
    pub fn decoding_key(&self) -> &DecodingKey {
        &self.decoding
    }
}

/// Estado compartido: pool de PostgreSQL y configuración JWT.
#[derive(Clone)]
pub struct AppState {
    /// Pool de conexiones a PostgreSQL.
    pub pool: PgPool,
    /// Parámetros de tokens de acceso y refresh.
    pub jwt: JwtConfig,
}

fn env_i64(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
