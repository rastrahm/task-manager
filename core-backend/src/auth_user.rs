//! Usuario autenticado extraído del header `Authorization: Bearer`.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};

use crate::app_config::AppState;
use crate::jwt::validate_access_token;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub is_admin: bool,
}

impl AuthUser {
    pub fn require_admin(&self) -> Result<(), StatusCode> {
        if self.is_admin {
            Ok(())
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }

    pub fn require_self_or_admin(&self, user_id: i32) -> Result<(), StatusCode> {
        if self.is_admin || self.id == user_id {
            Ok(())
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let claims = validate_access_token(&state.jwt, token).map_err(|_| StatusCode::UNAUTHORIZED)?;

        let active: Option<bool> = sqlx::query_scalar("SELECT is_active FROM users WHERE id = $1")
            .bind(claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        match active {
            Some(true) => Ok(AuthUser {
                id: claims.sub,
                username: claims.username,
                is_admin: claims.is_admin,
            }),
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
