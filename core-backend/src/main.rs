use axum::{
    routing::{get, post},
    extract::{State, Path},
    Json, Router,
};
use sqlx::postgres::{PgPool, PgPoolOptions};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

/// Represents a task in the system, as stored in the database.
/// Includes an optional `parent_id` for hierarchical tasks (subtasks).
#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct Task {
    id: i32,
    title: String,
    description: Option<String>,
    completed: bool,
    metadata: serde_json::Value,
    parent_id: Option<i32>,
}

/// Represents the data required to create a new task.
/// `title` is mandatory, while `description`, `metadata`, and `parent_id` are optional.
/// `parent_id` can be used to create a subtask.
#[derive(Deserialize)]
struct CreateTask {
    title: String,
    description: Option<String>,
    metadata: Option<serde_json::Value>,
    parent_id: Option<i32>,
}

type AppState = PgPool;

#[tokio::main]
async fn main() {
    // Conexión explícita a la base de datos
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgre@localhost/tasks_db".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("No se pudo conectar a PostgreSQL");

    let app = Router::new()
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/:id/toggle", post(toggle_task))
        .layer(CorsLayer::permissive()) // Permitir accesos de React, Android y Tauri
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Core Backend corriendo en http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Handles the GET request to retrieve all tasks from the database.
/// Tasks are ordered by their ID in descending order.
/// Returns a JSON array of Task objects.
async fn get_tasks(State(pool): State<AppState>) -> Json<Vec<Task>> {
    let tasks = sqlx::query_as::<_, Task>("SELECT id, title, description, completed, metadata, parent_id FROM tasks ORDER BY id DESC")
        .fetch_all(&pool)
        .await
        .unwrap();
    Json(tasks)
}

/// Handles the POST request to create a new task.
/// Expects a JSON payload of CreateTask and inserts it into the database.
/// The `metadata` field defaults to an empty JSON object if not provided.
/// The `parent_id` can be optionally provided to create a subtask.
/// Returns the newly created Task object as JSON.
async fn create_task(State(pool): State<AppState>, Json(payload): Json<CreateTask>) -> Json<Task> {
    let metadata = payload.metadata.unwrap_or(serde_json::json!({}));
    let task = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (title, description, metadata, parent_id) VALUES ($1, $2, $3, $4) RETURNING id, title, description, completed, metadata, parent_id"
    )
    .bind(payload.title)
    .bind(payload.description)
    .bind(metadata)
    .bind(payload.parent_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    Json(task)
}

/// Handles the POST request to toggle the 'completed' status of a task.
/// Takes the task ID from the URL path.
/// Returns `true` as JSON upon successful update.
async fn toggle_task(State(pool): State<AppState>, Path(id): Path<i32>) -> Json<bool> {
    sqlx::query("UPDATE tasks SET completed = NOT completed WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    Json(true)
}
