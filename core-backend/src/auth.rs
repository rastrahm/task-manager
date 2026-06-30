//! Login, renovación y cierre de sesión.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::app_config::AppState;
use crate::jwt::create_access_token;
use crate::refresh_token::{
    find_valid_refresh_token, generate_refresh_token, revoke_refresh_token,
    revoke_refresh_token_by_value, store_refresh_token,
};
use crate::users::{fetch_user_by_username, fetch_user_row, verify_user_password, UserResponse};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
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

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<StatusCode, StatusCode> {
    revoke_refresh_token_by_value(&state.pool, &payload.refresh_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}
