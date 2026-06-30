//! Usuario autenticado extraído del header `Authorization: Bearer`.
//!
//! [`AuthUser`] implementa [`FromRequestParts`]
//! y se usa como parámetro en handlers protegidos. Valida el JWT, comprueba
//! `token_type == "access"` y verifica que la cuenta siga activa en BD.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};

use crate::app_config::AppState;
use crate::jwt::validate_access_token;

/// Identidad del caller autenticado, derivada del access token JWT.
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// ID del usuario (`sub` en el JWT).
    pub id: i32,
    /// Nombre de usuario del token.
    pub username: String,
    /// Si el usuario tiene permisos de administrador.
    pub is_admin: bool,
}

impl AuthUser {
    /// Exige rol administrador; devuelve `403 Forbidden` si no lo tiene.
    pub fn require_admin(&self) -> Result<(), StatusCode> {
        if self.is_admin {
            Ok(())
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }

    /// Permite acceso al propio usuario o a cualquier ID si es admin.
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
