//! Cliente HTTP con JWT, renovación automática y persistencia de sesión.
//!
//! [`ApiClient`] centraliza login, refresh, logout y peticiones autenticadas
//! al backend. Renueva el access token antes de que expire y reintenta una vez
//! ante respuestas `401 Unauthorized`.

use crate::config::api_base_url;
use crate::models::{
    AuthResponse, CreateUserRequest, LoginRequest, LogoutRequest, RefreshRequest, Session,
    UpdateUserRequest, User,
};
use crate::session_store::{clear_session, load_session, save_session};
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;

const HTTP_TIMEOUT: Duration = Duration::from_secs(15);

/// Cliente del API REST con sesión JWT en memoria y en disco.
pub struct ApiClient {
    http: Client,
    session: Mutex<Option<Session>>,
}

impl ApiClient {
    /// Crea un cliente HTTP con timeout de 15 segundos y sin sesión activa.
    pub fn new() -> Self {
        let http = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .expect("cliente HTTP");

        Self {
            http,
            session: Mutex::new(None),
        }
    }

    /// Nombre del usuario de la sesión actual, si existe.
    pub fn username(&self) -> Option<String> {
        self.session
            .lock()
            .ok()
            .and_then(|session| session.as_ref().map(|s| s.user.username.clone()))
    }

    /// Indica si el usuario autenticado tiene rol de administrador.
    pub fn is_admin(&self) -> bool {
        self.session
            .lock()
            .ok()
            .and_then(|session| session.as_ref().map(|s| s.user.is_admin))
            .unwrap_or(false)
    }

    fn set_session(&self, session: Session) -> Result<(), String> {
        save_session(&session)?;
        if let Ok(mut guard) = self.session.lock() {
            *guard = Some(session);
        }
        Ok(())
    }

    /// Restaura la sesión desde disco (`session.json`) si existe.
    ///
    /// Devuelve `true` si se cargó una sesión válida en memoria.
    pub fn load_stored_session(&self) -> bool {
        let Some(stored) = load_session() else {
            return false;
        };
        if let Ok(mut guard) = self.session.lock() {
            *guard = Some(stored);
            return true;
        }
        false
    }

    /// Renueva los tokens si el access token expira en menos de 60 segundos.
    pub async fn refresh_session_if_needed(&self) -> Result<(), String> {
        let expiring = self
            .session
            .lock()
            .ok()
            .and_then(|session| session.as_ref().map(|s| s.is_access_expiring_soon()))
            .unwrap_or(false);

        if !expiring {
            return Ok(());
        }
        self.refresh_tokens().await
    }

    /// Autentica con usuario y contraseña; persiste la sesión en disco.
    pub async fn login(&self, username: &str, password: &str) -> Result<(), String> {
        let response = self
            .http
            .post(format!("{}/auth/login", api_base_url()))
            .json(&LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .map_err(|e| format!("No se pudo conectar con el backend ({API_TIMEOUT_SECS}s máx.): {e}"))?;

        if !response.status().is_success() {
            return Err(login_error_message(response).await);
        }

        let auth: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Respuesta de login inválida: {e}"))?;

        self.set_session(Session::from_auth(auth))
    }

    /// Intercambia el refresh token por un par de tokens nuevo.
    pub async fn refresh_tokens(&self) -> Result<(), String> {
        let refresh_token = self
            .session
            .lock()
            .ok()
            .and_then(|session| session.as_ref().map(|s| s.refresh_token.clone()))
            .ok_or("No hay sesión activa")?;

        let response = self
            .http
            .post(format!("{}/auth/refresh", api_base_url()))
            .json(&RefreshRequest { refresh_token })
            .send()
            .await
            .map_err(|e| format!("No se pudo renovar la sesión: {e}"))?;

        if !response.status().is_success() {
            self.clear_local_session();
            return Err("La sesión expiró; vuelve a iniciar sesión".to_string());
        }

        let auth: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Respuesta de renovación inválida: {e}"))?;

        self.set_session(Session::from_auth(auth))
    }

    /// Revoca el refresh token en el servidor y borra la sesión local.
    pub async fn logout(&self) -> Result<(), String> {
        let refresh_token = self
            .session
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|s| s.refresh_token.clone()));

        if let Some(refresh_token) = refresh_token {
            let _ = self
                .http
                .post(format!("{}/auth/logout", api_base_url()))
                .json(&LogoutRequest { refresh_token })
                .send()
                .await;
        }
        self.clear_local_session();
        Ok(())
    }

    /// Elimina tokens y usuario de memoria y del almacenamiento local.
    pub fn clear_local_session(&self) {
        clear_session();
        if let Ok(mut guard) = self.session.lock() {
            *guard = None;
        }
    }

    /// Petición GET autenticada; deserializa el cuerpo JSON a `T`.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        self.request_json(Method::GET, path, None::<&()>).await
    }

    /// Petición POST autenticada con cuerpo JSON.
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, String> {
        self.request_json(Method::POST, path, Some(body)).await
    }

    /// Petición POST autenticada sin cuerpo.
    pub async fn post_empty<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        self.request_json(Method::POST, path, None::<&()>).await
    }

    /// Petición PUT autenticada con cuerpo JSON.
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, String> {
        self.request_json(Method::PUT, path, Some(body)).await
    }

    /// Petición DELETE autenticada.
    pub async fn delete(&self, path: &str) -> Result<(), String> {
        let response = self.send_with_auth(Method::DELETE, path, None::<&()>).await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("El servidor respondió con error: {}", response.status()))
        }
    }

    async fn request_json<T, B>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, String>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let response = self.send_with_auth(method, path, body).await?;
        if !response.status().is_success() {
            return Err(format!("El servidor respondió con error: {}", response.status()));
        }
        response
            .json()
            .await
            .map_err(|e| format!("No se pudo interpretar la respuesta: {e}"))
    }

    async fn send_with_auth<B: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<Response, String> {
        let expiring = self
            .session
            .lock()
            .ok()
            .and_then(|session| session.as_ref().map(|s| s.is_access_expiring_soon()))
            .unwrap_or(false);

        if expiring {
            self.refresh_tokens().await?;
        }

        let response = self.send_once(method.clone(), path, body).await?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            self.refresh_tokens().await?;
            let retry = self.send_once(method, path, body).await?;
            if retry.status() == reqwest::StatusCode::UNAUTHORIZED {
                self.clear_local_session();
                return Err("Sesión no válida; vuelve a iniciar sesión".to_string());
            }
            return Ok(retry);
        }
        Ok(response)
    }

    async fn send_once<B: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<Response, String> {
        let access_token = self
            .session
            .lock()
            .ok()
            .and_then(|session| session.as_ref().map(|s| s.access_token.clone()))
            .ok_or("No hay sesión activa")?;

        let url = format!("{}{path}", api_base_url());
        let mut request = self
            .http
            .request(method, url)
            .bearer_auth(access_token);

        if let Some(payload) = body {
            request = request.json(payload);
        }

        request
            .send()
            .await
            .map_err(|e| format!("Error de red: {e}"))
    }

    /// Lista todos los usuarios (requiere rol administrador).
    pub async fn list_users(&self) -> Result<Vec<User>, String> {
        self.get("/users").await
    }

    /// Crea un usuario (requiere rol administrador).
    pub async fn create_user(&self, payload: &CreateUserRequest) -> Result<User, String> {
        self.post("/users", payload).await
    }

    /// Actualiza un usuario por id (requiere rol administrador).
    pub async fn update_user(&self, id: i32, payload: &UpdateUserRequest) -> Result<User, String> {
        self.put(&format!("/users/{id}"), payload).await
    }

    /// Elimina un usuario por id (requiere rol administrador).
    pub async fn delete_user(&self, id: i32) -> Result<(), String> {
        self.delete(&format!("/users/{id}")).await
    }
}

const API_TIMEOUT_SECS: u64 = HTTP_TIMEOUT.as_secs();

async fn login_error_message(response: Response) -> String {
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        "Usuario o contraseña incorrectos".to_string()
    } else {
        format!("Error de autenticación: {}", response.status())
    }
}
