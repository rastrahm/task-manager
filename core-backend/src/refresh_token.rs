//! Persistencia de refresh tokens (hash + rotación).

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct RefreshRecord {
    pub id: i32,
    pub user_id: i32,
}

pub fn hash_refresh_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!("{:x}", digest)
}

pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

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

pub async fn revoke_refresh_token(pool: &PgPool, token_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;
    Ok(())
}

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
