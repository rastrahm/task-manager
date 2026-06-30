//! Persistencia de refresh tokens (hash SHA-256 + rotación).
//!
//! El valor en claro del refresh token solo existe en el cliente y en tránsito;
//! en PostgreSQL se guarda [`hash_refresh_token`] del token. Cada refresh revoca
//! el token anterior y emite uno nuevo.

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// Fila mínima de un refresh token válido en BD.
#[derive(sqlx::FromRow)]
pub struct RefreshRecord {
    /// ID de la fila en `refresh_tokens`.
    pub id: i32,
    /// Usuario dueño de la sesión.
    pub user_id: i32,
}

/// Hash hexadecimal SHA-256 del refresh token (para almacenamiento seguro).
pub fn hash_refresh_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!("{:x}", digest)
}

/// Genera un refresh token opaco (UUID v4 en texto).
pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

/// Inserta un refresh token hasheado con fecha de expiración.
pub async fn store_refresh_token(
    pool: &PgPool,
    user_id: i32,
    token: &str,
    ttl_secs: i64,
) -> Result<(), sqlx::Error> {
    let expires_at = Utc::now() + Duration::seconds(ttl_secs);
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(hash_refresh_token(token))
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Busca un refresh token no revocado y no expirado.
pub async fn find_valid_refresh_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<RefreshRecord>, sqlx::Error> {
    let token_hash = hash_refresh_token(token);
    sqlx::query_as::<_, RefreshRecord>(
        "SELECT id, user_id FROM refresh_tokens
         WHERE token_hash = $1
           AND revoked_at IS NULL
           AND expires_at > NOW()",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}

/// Marca un refresh token como revocado por ID de fila.
pub async fn revoke_refresh_token(pool: &PgPool, token_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Revoca un refresh token por su valor en claro (p. ej. en logout).
pub async fn revoke_refresh_token_by_value(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW()
         WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(hash_refresh_token(token))
    .execute(pool)
    .await?;
    Ok(())
}

/// Revoca todas las sesiones activas de un usuario.
pub async fn revoke_all_user_refresh_tokens(pool: &PgPool, user_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW()
         WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_refresh_token_is_deterministic_sha256_hex() {
        let hash = hash_refresh_token("my-refresh-token");
        assert_eq!(hash, hash_refresh_token("my-refresh-token"));
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_refresh_token_differs_for_different_inputs() {
        assert_ne!(
            hash_refresh_token("token-a"),
            hash_refresh_token("token-b")
        );
    }

    #[test]
    fn generate_refresh_token_produces_unique_values() {
        let a = generate_refresh_token();
        let b = generate_refresh_token();
        assert_ne!(a, b);
        assert!(!a.is_empty());
    }
}
