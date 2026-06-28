//! API REST del gestor de tareas.
//!
//! Expone operaciones CRUD sobre tareas almacenadas en PostgreSQL, con soporte
//! para jerarquía padre/hijo (`parent_id`) y campos flexibles en `metadata` (JSONB).
//!
//! ## Endpoints
//!
//! | Método  | Ruta                        | Descripción                          |
//! |---------|-----------------------------|--------------------------------------|
//! | GET     | `/tasks`                    | Lista tareas raíz con `children`     |
//! | POST    | `/tasks`                    | Crea tarea o subtarea                |
//! | PUT     | `/tasks/:id`                | Reemplaza todos los campos editables |
//! | PATCH   | `/tasks/:id/description`    | Actualiza solo `description`         |
//! | PATCH   | `/tasks/:id/metadata`       | Actualiza solo `metadata`            |
//! | POST    | `/tasks/:id/toggle`         | Alterna el estado `completed`        |

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

/// Tarea tal como se persiste en la tabla `tasks`.
/// Corresponde fila a fila con el esquema definido en `init.sql`.
#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct Task {
    id: i32,
    title: String,
    description: Option<String>,
    completed: bool,
    metadata: serde_json::Value,
    /// Referencia a otra tarea; `None` indica tarea raíz.
    parent_id: Option<i32>,
}

/// Tarea devuelta por `GET /tasks`: mismos campos que `Task` más subtareas anidadas.
#[derive(Serialize)]
struct TaskTree {
    id: i32,
    title: String,
    description: Option<String>,
    completed: bool,
    metadata: serde_json::Value,
    parent_id: Option<i32>,
    children: Vec<TaskTree>,
}

/// Convierte la lista plana de la BD en un bosque de tareas raíz con `children` anidados.
///
/// Las tareas cuyo `parent_id` no existe en el conjunto cargado se tratan como raíz
/// para evitar nodos huérfanos en la respuesta JSON.
fn build_task_tree(tasks: Vec<Task>) -> Vec<TaskTree> {
    use std::collections::HashSet;

    let ids: HashSet<i32> = tasks.iter().map(|task| task.id).collect();
    let mut by_parent: HashMap<Option<i32>, Vec<&Task>> = HashMap::new();

    for task in &tasks {
        // Solo agrupa bajo un padre si ese padre está presente en el lote cargado.
        let parent_key = match task.parent_id {
            Some(parent_id) if ids.contains(&parent_id) => Some(parent_id),
            _ => None,
        };
        by_parent.entry(parent_key).or_default().push(task);
    }

    fn build_subtree(
        parent_id: Option<i32>,
        by_parent: &HashMap<Option<i32>, Vec<&Task>>,
    ) -> Vec<TaskTree> {
        let mut siblings: Vec<&Task> = by_parent.get(&parent_id).cloned().unwrap_or_default();
        siblings.sort_by_key(|task| task.id);

        siblings
            .into_iter()
            .map(|task| TaskTree {
                id: task.id,
                title: task.title.clone(),
                description: task.description.clone(),
                completed: task.completed,
                metadata: task.metadata.clone(),
                parent_id: task.parent_id,
                children: build_subtree(Some(task.id), by_parent),
            })
            .collect()
    }

    let mut roots = build_subtree(None, &by_parent);
    // Las raíces más recientes primero (id descendente).
    roots.sort_by_key(|task| std::cmp::Reverse(task.id));
    roots
}

/// Cuerpo esperado en `POST /tasks`.
/// `title` es obligatorio; el resto de campos son opcionales.
#[derive(Deserialize)]
struct CreateTask {
    title: String,
    description: Option<String>,
    /// Si no se envía, el backend guarda `{}`.
    metadata: Option<serde_json::Value>,
    /// Si se indica, la tarea se crea como subtarea del padre referenciado.
    parent_id: Option<i32>,
}

/// Cuerpo esperado en `PUT /tasks/:id` (reemplazo completo del recurso).
/// Todos los campos editables deben enviarse en cada petición.
#[derive(Deserialize)]
struct UpdateTask {
    title: String,
    description: Option<String>,
    completed: bool,
    metadata: serde_json::Value,
    parent_id: Option<i32>,
}

/// Cuerpo esperado en `PATCH /tasks/:id/description`.
/// Enviar `"description": null` elimina el texto de la descripción.
#[derive(Deserialize)]
struct PatchDescription {
    description: Option<String>,
}

/// Cuerpo esperado en `PATCH /tasks/:id/metadata`.
/// Reemplaza por completo el objeto JSON almacenado en la columna `metadata`.
#[derive(Deserialize)]
struct PatchMetadata {
    metadata: serde_json::Value,
}

/// Estado compartido de la aplicación: pool de conexiones a PostgreSQL.
type AppState = PgPool;

/// Consulta base reutilizada por los handlers que leen o devuelven una tarea.
const TASK_SELECT: &str =
    "SELECT id, title, description, completed, metadata, parent_id FROM tasks";

/// Obtiene una tarea por `id` o devuelve `404 Not Found`.
async fn fetch_task(pool: &PgPool, id: i32) -> Result<Task, StatusCode> {
    sqlx::query_as::<_, Task>(&format!("{TASK_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}

/// Impide que una tarea sea su propio padre (`parent_id == id`).
fn validate_parent_id(id: i32, parent_id: Option<i32>) -> Result<(), StatusCode> {
    if parent_id == Some(id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    // `DATABASE_URL` permite apuntar a distintos entornos sin recompilar.
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgre@localhost/tasks_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("No se pudo conectar a PostgreSQL");

    let app = Router::new()
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id/description", patch(patch_description))
        .route("/tasks/:id/metadata", patch(patch_metadata))
        .route("/tasks/:id/toggle", post(toggle_task))
        // CORS permisivo para clientes web, móvil y escritorio en desarrollo.
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 5040));
    println!("Core Backend corriendo en http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// `GET /tasks` — Devuelve las tareas raíz con sus subtareas en `children`.
async fn get_tasks(State(pool): State<AppState>) -> Json<Vec<TaskTree>> {
    let tasks = sqlx::query_as::<_, Task>(&format!("{TASK_SELECT} ORDER BY id ASC"))
        .fetch_all(&pool)
        .await
        .unwrap();
    Json(build_task_tree(tasks))
}

/// `POST /tasks` — Inserta una tarea nueva y devuelve el registro creado.
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

/// `PUT /tasks/:id` — Reemplaza título, descripción, estado, metadata y padre.
///
/// Respuestas: `200` con la tarea actualizada, `404` si no existe,
/// `400` si `parent_id` apunta a la misma tarea.
async fn update_task(
    State(pool): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTask>,
) -> Result<Json<Task>, StatusCode> {
    validate_parent_id(id, payload.parent_id)?;
    fetch_task(&pool, id).await?;

    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET title = $1, description = $2, completed = $3, metadata = $4, parent_id = $5
         WHERE id = $6
         RETURNING id, title, description, completed, metadata, parent_id",
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.completed)
    .bind(&payload.metadata)
    .bind(payload.parent_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task))
}

/// `PATCH /tasks/:id/description` — Actualiza únicamente el campo `description`.
async fn patch_description(
    State(pool): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<PatchDescription>,
) -> Result<Json<Task>, StatusCode> {
    fetch_task(&pool, id).await?;

    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET description = $1
         WHERE id = $2
         RETURNING id, title, description, completed, metadata, parent_id",
    )
    .bind(&payload.description)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task))
}

/// `PATCH /tasks/:id/metadata` — Actualiza únicamente el objeto JSON `metadata`.
async fn patch_metadata(
    State(pool): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<PatchMetadata>,
) -> Result<Json<Task>, StatusCode> {
    fetch_task(&pool, id).await?;

    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET metadata = $1
         WHERE id = $2
         RETURNING id, title, description, completed, metadata, parent_id",
    )
    .bind(&payload.metadata)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task))
}

/// `POST /tasks/:id/toggle` — Invierte el valor de `completed` sin modificar otros campos.
async fn toggle_task(State(pool): State<AppState>, Path(id): Path<i32>) -> Json<bool> {
    sqlx::query("UPDATE tasks SET completed = NOT completed WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    Json(true)
}
