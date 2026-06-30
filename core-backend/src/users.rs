//! Modelo de usuario y CRUD.
//!
//! Todas las operaciones HTTP usan el mismo [`UserDto`]. El campo `password`
//! solo se acepta en creación/actualización y nunca se serializa en respuestas.
//!
//! Reglas de autorización:
//! - **Admin**: listar, crear y eliminar usuarios; ver y editar cualquier perfil.
//! - **Usuario normal**: solo puede leer y actualizar su propio registro.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::app_config::AppState;
use crate::auth_user::AuthUser;
use crate::password::{hash_password, verify_password};

/// DTO único de usuario para listar, crear, actualizar y respuestas de auth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub username: Option<String>,
    /// Solo entrada (create/update). Nunca aparece en JSON de salida.
    #[serde(skip_serializing, default)]
    pub password: Option<String>,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(sqlx::FromRow)]
pub(crate) struct UserRow {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UserDto {
    pub(crate) fn from_row(row: UserRow) -> Self {
        Self {
            id: Some(row.id),
            username: Some(row.username),
            password: None,
            is_admin: Some(row.is_admin),
            is_active: Some(row.is_active),
            created_at: Some(row.created_at),
            updated_at: Some(row.updated_at),
        }
    }
}

const USER_SELECT: &str =
    "SELECT id, username, password_hash, is_admin, is_active, created_at, updated_at FROM users";

/// `GET /users` — lista todos los usuarios (solo admin).
pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<UserDto>>, StatusCode> {
    auth.require_admin()?;

    let users = sqlx::query_as::<_, UserRow>(&format!("{USER_SELECT} ORDER BY id ASC"))
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users.into_iter().map(UserDto::from_row).collect()))
}

/// `GET /users/:id` — perfil propio o cualquiera si es admin.
pub async fn get_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<UserDto>, StatusCode> {
    auth.require_self_or_admin(id)?;
    Ok(Json(UserDto::from_row(fetch_user_row(&state.pool, id).await?)))
}

/// `POST /users` — crea un usuario (solo admin). `409` si el username ya existe.
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<UserDto>,
) -> Result<Json<UserDto>, StatusCode> {
    auth.require_admin()?;

    let username = payload
        .username
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let password = payload
        .password
        .as_deref()
        .filter(|p| !p.is_empty())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let password_hash = hash_password(password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let is_admin = payload.is_admin.unwrap_or(false);

    let user = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (username, password_hash, is_admin)
         VALUES ($1, $2, $3)
         RETURNING id, username, password_hash, is_admin, is_active, created_at, updated_at",
    )
    .bind(username)
    .bind(password_hash)
    .bind(is_admin)
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        if let sqlx::Error::Database(db) = &error {
            if db.constraint().is_some() {
                return StatusCode::CONFLICT;
            }
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(UserDto::from_row(user)))
}

/// `PUT /users/:id` — actualiza perfil. Solo admin puede cambiar `is_admin` / `is_active`.
pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<UserDto>,
) -> Result<Json<UserDto>, StatusCode> {
    auth.require_self_or_admin(id)?;

    let existing = fetch_user_row(&state.pool, id).await?;

    let username = payload
        .username
        .unwrap_or(existing.username)
        .trim()
        .to_string();
    if username.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let is_admin = if auth.is_admin {
        payload.is_admin.unwrap_or(existing.is_admin)
    } else {
        existing.is_admin
    };

    let is_active = if auth.is_admin {
        payload.is_active.unwrap_or(existing.is_active)
    } else {
        existing.is_active
    };

    let password_hash = if let Some(password) = payload.password {
        if password.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        hash_password(&password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        existing.password_hash
    };

    let user = sqlx::query_as::<_, UserRow>(
        "UPDATE users
         SET username = $1, password_hash = $2, is_admin = $3, is_active = $4, updated_at = NOW()
         WHERE id = $5
         RETURNING id, username, password_hash, is_admin, is_active, created_at, updated_at",
    )
    .bind(username)
    .bind(password_hash)
    .bind(is_admin)
    .bind(is_active)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UserDto::from_row(user)))
}

/// `DELETE /users/:id` — elimina usuario (solo admin). No permite auto-eliminación.
pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    auth.require_admin()?;

    if auth.id == id {
        return Err(StatusCode::BAD_REQUEST);
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Carga una fila de usuario por ID (incluye hash de contraseña).
pub(crate) async fn fetch_user_row(pool: &sqlx::PgPool, id: i32) -> Result<UserRow, StatusCode> {
    sqlx::query_as::<_, UserRow>(&format!("{USER_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}

/// Busca usuario por nombre para login. `401` si no existe.
pub(crate) async fn fetch_user_by_username(
    pool: &sqlx::PgPool,
    username: &str,
) -> Result<UserRow, StatusCode> {
    sqlx::query_as::<_, UserRow>(&format!("{USER_SELECT} WHERE username = $1"))
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

/// Verifica la contraseña de un usuario interno con Argon2id.
pub(crate) fn verify_user_password(user: &UserRow, password: &str) -> bool {
    verify_password(password, &user.password_hash)
}

/// Si `admin` tiene `password_hash = 'PENDING_HASH'`, lo sustituye al arrancar.
pub async fn ensure_admin_password(pool: &sqlx::PgPool) {
    let pending: Option<String> = sqlx::query_scalar(
        "SELECT password_hash FROM users WHERE username = 'admin' AND password_hash = 'PENDING_HASH'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if pending.is_none() {
        return;
    }

    let initial = std::env::var("ADMIN_INITIAL_PASSWORD").unwrap_or_else(|_| "changeme".to_string());
    let Ok(hash) = hash_password(&initial) else {
        return;
    };

    let _ = sqlx::query(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE username = 'admin'",
    )
    .bind(hash)
    .execute(pool)
    .await;

    println!(
        "Admin inicial configurado: usuario 'admin', contraseña desde ADMIN_INITIAL_PASSWORD (default: changeme)"
    );
}
