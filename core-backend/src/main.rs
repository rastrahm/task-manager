//! API REST del gestor de tareas con autenticación JWT y CRUD de usuarios.

use axum::{
    routing::{get, patch, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod app_config;
mod auth;
mod auth_user;
mod db_config;
mod jwt;
mod password;
mod refresh_token;
mod tasks;
mod users;

use app_config::{AppState, JwtConfig};
use auth::{login, logout, refresh};
use tasks::{
    create_task, get_tasks, patch_description, patch_metadata, toggle_task, update_task,
};
use users::{create_user, delete_user, get_user, list_users, update_user};

#[tokio::main]
async fn main() {
    let pool = db_config::DbConfig::connect_pool(5).await;
    users::ensure_admin_password(&pool).await;

    let state = AppState {
        pool,
        jwt: JwtConfig::from_env(),
    };

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

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 5040));
    println!("Core Backend corriendo en http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
