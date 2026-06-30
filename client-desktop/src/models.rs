//! Modelos compartidos de autenticación y usuarios.
//!
//! Tipos serializables que reflejan las respuestas y peticiones del backend
//! en los endpoints `/auth/*` y `/users`.

use serde::{Deserialize, Serialize};

/// Usuario devuelto por el API (sin hash de contraseña).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Respuesta de login o refresh con tokens y datos del usuario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    /// Segundos hasta la caducidad del access token.
    pub expires_in: i64,
    pub user: User,
}

/// Sesión persistida localmente: tokens, caducidad y usuario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    /// Marca de tiempo Unix (UTC) en la que expira el access token.
    pub expires_at: i64,
    pub user: User,
}

impl Session {
    /// Construye una sesión a partir de la respuesta de autenticación del backend.
    pub fn from_auth(auth: AuthResponse) -> Self {
        let expires_at = chrono::Utc::now().timestamp() + auth.expires_in;
        Self {
            access_token: auth.access_token,
            refresh_token: auth.refresh_token,
            expires_at,
            user: auth.user,
        }
    }

    /// `true` si el access token caduca en menos de 60 segundos.
    pub fn is_access_expiring_soon(&self) -> bool {
        chrono::Utc::now().timestamp() >= self.expires_at - 60
    }
}

/// Cuerpo de `POST /auth/login`.
#[derive(Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Cuerpo de `POST /auth/refresh`.
#[derive(Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Cuerpo de `POST /auth/logout`.
#[derive(Serialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

/// Cuerpo de `POST /users` (solo administrador).
#[derive(Serialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

/// Cuerpo de `PUT /users/:id`; los campos `None` no se envían.
#[derive(Serialize)]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}
