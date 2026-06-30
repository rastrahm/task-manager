//! Login, renovación y cierre de sesión.
//!
//! Las rutas de este módulo son **públicas** (no requieren `Authorization`).
//! Tras un login o refresh exitoso se devuelve un par access + refresh token;
//! el refresh se rota en cada llamada a [`refresh`].

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::app_config::AppState;
use crate::jwt::create_access_token;
use crate::refresh_token::{
    find_valid_refresh_token, generate_refresh_token, revoke_refresh_token,
    revoke_refresh_token_by_value, store_refresh_token,
};
use crate::users::{fetch_user_by_username, fetch_user_row, verify_user_password, UserResponse};

/// Cuerpo de `POST /auth/login`.
#[derive(Deserialize)]
pub struct LoginRequest {
    /// Nombre de usuario (se recorta espacios al inicio y al final).
    pub username: String,
    /// Contraseña en texto plano (solo en tránsito; nunca se almacena).
    pub password: String,
}

/// Cuerpo de `POST /auth/refresh`.
#[derive(Deserialize)]
pub struct RefreshRequest {
    /// Refresh token emitido en el login o refresh anterior.
    pub refresh_token: String,
}

/// Cuerpo de `POST /auth/logout`.
#[derive(Deserialize)]
pub struct LogoutRequest {
    /// Refresh token a revocar en base de datos.
    pub refresh_token: String,
}

/// Respuesta de login y refresh.
#[derive(Serialize)]
pub struct AuthResponse {
    /// JWT de acceso para el header `Authorization: Bearer`.
    pub access_token: String,
    /// Token opaco para renovar la sesión (guardar de forma segura en el cliente).
    pub refresh_token: String,
    /// Siempre `"Bearer"`.
    pub token_type: &'static str,
    /// Segundos hasta la caducidad del access token.
    pub expires_in: i64,
    /// Perfil del usuario autenticado (sin hash de contraseña).
    pub user: UserResponse,
}

async fn issue_tokens(state: &AppState, user_id: i32) -> Result<AuthResponse, StatusCode> {
    let user = fetch_user_row(&state.pool, user_id).await?;
    if !user.is_active {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let access_token = create_access_token(
        &state.jwt,
        user.id,
        &user.username,
        user.is_admin,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refresh_token = generate_refresh_token();
    store_refresh_token(
        &state.pool,
        user.id,
        &refresh_token,
        state.jwt.refresh_ttl_secs,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: state.jwt.access_ttl_secs,
        user: user.into(),
    })
}

/// `POST /auth/login` — autentica con usuario y contraseña.
///
/// # Errores
///
/// - `401` — credenciales incorrectas o cuenta inactiva.
/// - `500` — error al firmar JWT o persistir refresh token.
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let user = fetch_user_by_username(&state.pool, payload.username.trim()).await?;

    if !user.is_active || !verify_user_password(&user, &payload.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(Json(issue_tokens(&state, user.id).await?))
}

/// `POST /auth/refresh` — rota el refresh token y emite un par nuevo.
///
/// El refresh anterior queda revocado. Si el token no existe, expiró o ya fue usado,
/// responde `401`.
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    if payload.refresh_token.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let record = find_valid_refresh_token(&state.pool, &payload.refresh_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    revoke_refresh_token(&state.pool, record.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(issue_tokens(&state, record.user_id).await?))
}

/// `POST /auth/logout` — revoca el refresh token indicado.
///
/// Responde `204 No Content` aunque el token ya estuviera revocado.
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<StatusCode, StatusCode> {
    revoke_refresh_token_by_value(&state.pool, &payload.refresh_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}
