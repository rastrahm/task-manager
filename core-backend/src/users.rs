//! Modelo de usuario y CRUD.
//!
//! Reglas de autorización:
//! - **Admin**: listar, crear y eliminar usuarios; ver y editar cualquier perfil.
//! - **Usuario normal**: solo puede leer y actualizar su propio registro.
//!
//! [`ensure_admin_password`] se ejecuta al arrancar el servidor para bootstrap del admin.

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

/// Perfil de usuario expuesto en JSON (sin `password_hash`).
#[derive(Serialize)]
pub struct UserResponse {
    id: i32,
    username: String,
    is_admin: bool,
    is_active: bool,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<UserRow> for UserResponse {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            is_admin: row.is_admin,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Cuerpo de `POST /users` (solo admin).
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub is_admin: bool,
}

/// Cuerpo de `PUT /users/:id`. Todos los campos son opcionales.
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
}

const USER_SELECT: &str =
    "SELECT id, username, password_hash, is_admin, is_active, created_at, updated_at FROM users";

/// `GET /users` — lista todos los usuarios (solo admin).
pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    auth.require_admin()?;

    let users = sqlx::query_as::<_, UserRow>(&format!("{USER_SELECT} ORDER BY id ASC"))
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

/// `GET /users/:id` — perfil propio o cualquiera si es admin.
pub async fn get_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, StatusCode> {
    auth.require_self_or_admin(id)?;
    fetch_user_response(&state.pool, id).await
}

/// `POST /users` — crea un usuario (solo admin). `409` si el username ya existe.
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    auth.require_admin()?;

    if payload.username.trim().is_empty() || payload.password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let password_hash = hash_password(&payload.password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (username, password_hash, is_admin)
         VALUES ($1, $2, $3)
         RETURNING id, username, password_hash, is_admin, is_active, created_at, updated_at",
    )
    .bind(payload.username.trim())
    .bind(password_hash)
    .bind(payload.is_admin)
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

    Ok(Json(user.into()))
}

/// `PUT /users/:id` — actualiza perfil. Solo admin puede cambiar `is_admin` / `is_active`.
pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
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

    Ok(Json(user.into()))
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
pub async fn fetch_user_row(pool: &sqlx::PgPool, id: i32) -> Result<UserRow, StatusCode> {
    sqlx::query_as::<_, UserRow>(&format!("{USER_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}

async fn fetch_user_response(pool: &sqlx::PgPool, id: i32) -> Result<Json<UserResponse>, StatusCode> {
    Ok(Json(fetch_user_row(pool, id).await?.into()))
}

/// Busca usuario por nombre para login. `401` si no existe.
pub async fn fetch_user_by_username(
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
pub fn verify_user_password(user: &UserRow, password: &str) -> bool {
    verify_password(password, &user.password_hash)
}

/// Si `admin` tiene `password_hash = 'PENDING_HASH'`, lo sustituye al arrancar.
///
/// Usa `ADMIN_INITIAL_PASSWORD` (default `changeme`).
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
