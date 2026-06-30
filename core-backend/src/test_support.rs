//! Utilidades compartidas por tests de integración.
//!
//! Requiere PostgreSQL con el esquema de `init.sql` aplicado en la base
//! `tasks_db_test` (o el nombre en `TEST_DB_NAME`):
//!
//! ```bash
//! createdb tasks_db_test
//! psql -d tasks_db_test -f init.sql
//! ```

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tower::ServiceExt;

use crate::app_config::{AppState, JwtConfig};
use crate::build_app;
use crate::users;

const TEST_ADMIN_PASSWORD: &str = "testpass";

static DB_SETUP_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn db_setup_lock() -> &'static Mutex<()> {
    DB_SETUP_LOCK.get_or_init(|| Mutex::new(()))
}

/// Reinicia datos y deja al admin con contraseña conocida.
pub async fn reset_database(pool: &PgPool) {
    sqlx::query("TRUNCATE refresh_tokens, tasks, users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("TRUNCATE en base de pruebas");

    sqlx::query(
        "INSERT INTO users (username, password_hash, is_admin) VALUES ('admin', 'PENDING_HASH', TRUE)",
    )
    .execute(pool)
    .await
    .expect("insertar admin de prueba");

    std::env::set_var("ADMIN_INITIAL_PASSWORD", TEST_ADMIN_PASSWORD);
    users::ensure_admin_password(pool).await;
}

/// App de prueba con bloqueo de BD para evitar carreras entre tests paralelos.
pub struct TestApp {
    _db_guard: tokio::sync::MutexGuard<'static, ()>,
    router: Router,
}

impl std::ops::Deref for TestApp {
    type Target = Router;

    fn deref(&self) -> &Self::Target {
        &self.router
    }
}

/// Router Axum listo para pruebas HTTP.
pub async fn test_app() -> TestApp {
    let guard = db_setup_lock().lock().await;

    let pool = crate::db_config::DbConfig::connect_test_pool(5).await;
    reset_database(&pool).await;

    let state = AppState {
        pool,
        jwt: JwtConfig::for_tests(),
    };

    TestApp {
        _db_guard: guard,
        router: build_app(state),
    }
}

pub async fn post_json(
    app: &Router,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    request(app, Method::POST, uri, Some(body), bearer).await
}

pub async fn get_json(app: &Router, uri: &str, bearer: Option<&str>) -> (StatusCode, Value) {
    request(app, Method::GET, uri, None, bearer).await
}

pub async fn put_json(
    app: &Router,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    request(app, Method::PUT, uri, Some(body), bearer).await
}

pub async fn patch_json(
    app: &Router,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    request(app, Method::PATCH, uri, Some(body), bearer).await
}

pub async fn delete_request(
    app: &Router,
    uri: &str,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    request(app, Method::DELETE, uri, None, bearer).await
}

pub async fn login(app: &Router, username: &str, password: &str) -> Value {
    let (status, body) = post_json(
        app,
        "/auth/login",
        json!({ "username": username, "password": password }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "login falló: {body}");
    body
}

pub fn bearer_from_auth(auth: &Value) -> String {
    auth["access_token"]
        .as_str()
        .expect("access_token en respuesta de login")
        .to_string()
}

async fn request(
    app: &Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);

    if let Some(token) = bearer {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }

    let request = if let Some(json_body) = body {
        builder
            .header("content-type", "application/json")
            .body(Body::from(json_body.to_string()))
    } else {
        builder.body(Body::empty())
    }
    .expect("construir petición de prueba");

    let response = app.clone().oneshot(request).await.expect("respuesta HTTP");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("leer cuerpo")
        .to_bytes();

    let json_body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            panic!(
                "JSON inválido (status {status}): {} — {error}",
                String::from_utf8_lossy(&bytes)
            )
        })
    };

    (status, json_body)
}

pub fn admin_password() -> &'static str {
    TEST_ADMIN_PASSWORD
}
