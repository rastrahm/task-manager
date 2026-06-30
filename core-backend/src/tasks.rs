//! Handlers de tareas con aislamiento por `user_id`.
//!
//! - Cada usuario ve solo sus tareas.
//! - Los administradores ven todas las tareas del sistema.
//! - `GET /tasks` devuelve un **árbol**: raíces con subtareas en `children`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

use crate::app_config::AppState;
use crate::auth_user::AuthUser;

/// Fila de tarea en PostgreSQL (`user_id` no se serializa al cliente).
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: i32,
    #[serde(skip_serializing)]
    pub user_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub metadata: serde_json::Value,
    pub parent_id: Option<i32>,
}

/// Nodo del árbol devuelto por `GET /tasks`.
#[derive(Serialize)]
pub struct TaskTree {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub metadata: serde_json::Value,
    pub parent_id: Option<i32>,
    pub children: Vec<TaskTree>,
}

const TASK_SELECT: &str =
    "SELECT id, user_id, title, description, completed, metadata, parent_id FROM tasks";

/// Cuerpo de `POST /tasks`.
#[derive(Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub parent_id: Option<i32>,
}

/// Cuerpo de `PUT /tasks/:id`.
#[derive(Deserialize)]
pub struct UpdateTask {
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub metadata: serde_json::Value,
    pub parent_id: Option<i32>,
}

/// Cuerpo de `PATCH /tasks/:id/description`.
#[derive(Deserialize)]
pub struct PatchDescription {
    pub description: Option<String>,
}

/// Cuerpo de `PATCH /tasks/:id/metadata`.
#[derive(Deserialize)]
pub struct PatchMetadata {
    pub metadata: serde_json::Value,
}

/// Convierte una lista plana de tareas en un bosque de [`TaskTree`] anidados.
pub fn build_task_tree(tasks: Vec<Task>) -> Vec<TaskTree> {
    use std::collections::HashSet;

    let ids: HashSet<i32> = tasks.iter().map(|task| task.id).collect();
    let mut by_parent: HashMap<Option<i32>, Vec<&Task>> = HashMap::new();

    for task in &tasks {
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
    roots.sort_by_key(|task| std::cmp::Reverse(task.id));
    roots
}

async fn fetch_task_for_user(pool: &PgPool, id: i32, auth: &AuthUser) -> Result<Task, StatusCode> {
    let task = sqlx::query_as::<_, Task>(&format!("{TASK_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if !auth.is_admin && task.user_id != auth.id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(task)
}

fn validate_parent_id(id: i32, parent_id: Option<i32>) -> Result<(), StatusCode> {
    if parent_id == Some(id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

async fn validate_parent_belongs_to_user(
    pool: &PgPool,
    parent_id: i32,
    user_id: i32,
) -> Result<(), StatusCode> {
    let owner: Option<i32> = sqlx::query_scalar("SELECT user_id FROM tasks WHERE id = $1")
        .bind(parent_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match owner {
        Some(owner_id) if owner_id == user_id => Ok(()),
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::BAD_REQUEST),
    }
}

/// `GET /tasks` — árbol de tareas del usuario (o todas si es admin).
pub async fn get_tasks(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<TaskTree>>, StatusCode> {
    let tasks = if auth.is_admin {
        sqlx::query_as::<_, Task>(&format!("{TASK_SELECT} ORDER BY id ASC"))
            .fetch_all(&state.pool)
            .await
    } else {
        sqlx::query_as::<_, Task>(&format!("{TASK_SELECT} WHERE user_id = $1 ORDER BY id ASC"))
            .bind(auth.id)
            .fetch_all(&state.pool)
            .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(build_task_tree(tasks)))
}

/// `POST /tasks` — crea tarea o subtarea (`parent_id` opcional).
pub async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateTask>,
) -> Result<Json<Task>, StatusCode> {
    if payload.title.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user_id = auth.id;
    if let Some(parent_id) = payload.parent_id {
        validate_parent_belongs_to_user(&state.pool, parent_id, user_id).await?;
    }

    let metadata = payload.metadata.unwrap_or(serde_json::json!({}));
    let task = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (user_id, title, description, metadata, parent_id)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, user_id, title, description, completed, metadata, parent_id",
    )
    .bind(user_id)
    .bind(payload.title.trim())
    .bind(payload.description)
    .bind(metadata)
    .bind(payload.parent_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task))
}

/// `PUT /tasks/:id` — reemplaza título, descripción, estado, metadatos y padre.
pub async fn update_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTask>,
) -> Result<Json<Task>, StatusCode> {
    validate_parent_id(id, payload.parent_id)?;
    let existing = fetch_task_for_user(&state.pool, id, &auth).await?;

    if let Some(parent_id) = payload.parent_id {
        validate_parent_belongs_to_user(&state.pool, parent_id, existing.user_id).await?;
    }

    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET title = $1, description = $2, completed = $3, metadata = $4, parent_id = $5
         WHERE id = $6
         RETURNING id, user_id, title, description, completed, metadata, parent_id",
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.completed)
    .bind(&payload.metadata)
    .bind(payload.parent_id)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task))
}

/// `PATCH /tasks/:id/description` — actualiza solo la descripción.
pub async fn patch_description(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<PatchDescription>,
) -> Result<Json<Task>, StatusCode> {
    fetch_task_for_user(&state.pool, id, &auth).await?;

    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET description = $1
         WHERE id = $2
         RETURNING id, user_id, title, description, completed, metadata, parent_id",
    )
    .bind(&payload.description)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task))
}

/// `PATCH /tasks/:id/metadata` — actualiza solo el JSON de metadatos.
pub async fn patch_metadata(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<PatchMetadata>,
) -> Result<Json<Task>, StatusCode> {
    fetch_task_for_user(&state.pool, id, &auth).await?;

    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET metadata = $1
         WHERE id = $2
         RETURNING id, user_id, title, description, completed, metadata, parent_id",
    )
    .bind(&payload.metadata)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task))
}

/// `POST /tasks/:id/toggle` — invierte el campo `completed`.
pub async fn toggle_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<bool>, StatusCode> {
    fetch_task_for_user(&state.pool, id, &auth).await?;

    sqlx::query("UPDATE tasks SET completed = NOT completed WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(true))
}
