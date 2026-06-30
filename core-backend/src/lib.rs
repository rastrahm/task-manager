//! # task-core — API REST del gestor de tareas
//!
//! Backend HTTP escrito con [Axum](https://github.com/tokio-rs/axum) y PostgreSQL.
//! Gestiona usuarios, autenticación JWT con refresh tokens rotativos y tareas
//! jerárquicas por usuario.
//!
//! ## Inicio rápido
//!
//! ```bash
//! cd core-backend
//! cp .env.example .env
//! psql -d tasks_db -f init.sql
//! cargo run
//! ```
//!
//! El servidor escucha en `http://0.0.0.0:5040`.
//!
//! ## Configuración
//!
//! | Variable | Descripción | Default |
//! |----------|-------------|---------|
//! | `DB_HOST` | Host de PostgreSQL | `localhost` |
//! | `DB_PORT` | Puerto | `5432` |
//! | `DB_USER` | Usuario | `postgres` |
//! | `DB_PASSWORD` | Contraseña | `postgre` |
//! | `DB_NAME` | Base de datos | `tasks_db` |
//! | `JWT_SECRET` | Secreto HMAC para firmar JWT | *(solo desarrollo)* |
//! | `JWT_ACCESS_TTL_SECS` | Caducidad del access token (s) | `3600` |
//! | `JWT_REFRESH_TTL_SECS` | Caducidad del refresh token (s) | `604800` |
//! | `ADMIN_INITIAL_PASSWORD` | Contraseña inicial de `admin` | `changeme` |
//!
//! Al arrancar, si el usuario `admin` tiene `password_hash = 'PENDING_HASH'`,
//! se sustituye por un hash Argon2 de `ADMIN_INITIAL_PASSWORD`.
//!
//! ## Autenticación
//!
//! 1. `POST /auth/login` con `{ "username", "password" }` devuelve access + refresh.
//! 2. Las rutas protegidas exigen `Authorization: Bearer <access_token>`.
//! 3. `POST /auth/refresh` rota el refresh token y emite un par nuevo.
//! 4. `POST /auth/logout` revoca el refresh token enviado.
//!
//! Los access tokens incluyen `sub`, `username`, `is_admin` y `token_type: "access"`.
//!
//! ## API HTTP
//!
//! ### Rutas públicas
//!
//! | Método | Ruta | Handler |
//! |--------|------|---------|
//! | POST | `/auth/login` | [`auth::login`] |
//! | POST | `/auth/refresh` | [`auth::refresh`] |
//! | POST | `/auth/logout` | [`auth::logout`] |
//!
//! ### Rutas protegidas
//!
//! | Método | Ruta | Descripción |
//! |--------|------|-------------|
//! | GET | `/tasks` | Árbol de tareas ([`tasks::get_tasks`]) |
//! | POST | `/tasks` | Crear tarea ([`tasks::create_task`]) |
//! | PUT | `/tasks/:id` | Actualizar tarea ([`tasks::update_task`]) |
//! | PATCH | `/tasks/:id/description` | Solo descripción ([`tasks::patch_description`]) |
//! | PATCH | `/tasks/:id/metadata` | Solo metadatos ([`tasks::patch_metadata`]) |
//! | POST | `/tasks/:id/toggle` | Alternar `completed` ([`tasks::toggle_task`]) |
//! | GET | `/users` | Listar usuarios, admin ([`users::list_users`]) |
//! | POST | `/users` | Crear usuario, admin ([`users::create_user`]) |
//! | GET | `/users/:id` | Ver usuario propio o admin ([`users::get_user`]) |
//! | PUT | `/users/:id` | Actualizar usuario ([`users::update_user`]) |
//! | DELETE | `/users/:id` | Eliminar usuario, admin ([`users::delete_user`]) |
//!
//! ## Módulos
//!
//! - [`app_config`] — estado compartido y configuración JWT.
//! - [`auth`] — login, refresh y logout.
//! - [`auth_user`] — extractor Axum del usuario autenticado.
//! - [`db_config`] — conexión a PostgreSQL.
//! - [`jwt`] — emisión y validación de access tokens.
//! - [`password`] — hash Argon2id de contraseñas.
//! - [`refresh_token`] — persistencia y rotación de refresh tokens.
//! - [`tasks`] — CRUD de tareas con aislamiento por `user_id`.
//! - [`users`] — CRUD de usuarios y bootstrap del admin.
//!
//! ## Documentación local
//!
//! ```bash
//! cargo doc --open
//! ```

pub mod app_config;
pub mod auth;
pub mod auth_user;
pub mod db_config;
pub mod jwt;
pub mod password;
pub mod refresh_token;
pub mod tasks;
pub mod users;

use axum::{
    routing::{get, patch, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use app_config::{AppState, JwtConfig};
use auth::{login, logout, refresh};
use tasks::{
    create_task, get_tasks, patch_description, patch_metadata, toggle_task, update_task,
};
use users::{create_user, delete_user, get_user, list_users, update_user};

/// Puerto TCP por defecto del servidor HTTP.
pub const DEFAULT_PORT: u16 = 5040;

/// Construye el router Axum con todas las rutas y el estado compartido.
pub fn build_app(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout));

    let protected_routes = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id/description", patch(patch_description))
        .route("/tasks/:id/metadata", patch(patch_metadata))
        .route("/tasks/:id/toggle", post(toggle_task));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Inicializa la base de datos, configura el admin y arranca el servidor HTTP.
///
/// Conecta a PostgreSQL según [`db_config::DbConfig`], aplica
/// [`users::ensure_admin_password`] y escucha en `0.0.0.0:`[`DEFAULT_PORT`].
pub async fn serve() {
    let pool = db_config::DbConfig::connect_pool(5).await;
    users::ensure_admin_password(&pool).await;

    let state = AppState {
        pool,
        jwt: JwtConfig::from_env(),
    };

    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT));
    println!("Core Backend corriendo en http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
