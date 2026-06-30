//! Handlers de tareas con aislamiento por `user_id`.
//!
//! Todas las operaciones HTTP usan el mismo [`TaskDto`]. Los campos `Option` en
//! escritura significan «no enviado»; en lectura se devuelven siempre completos.
//! `GET /tasks` rellena `children` con el árbol de subtareas.

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

/// DTO único de tarea para listar, crear, actualizar y parchear.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    /// Presente en respuestas; debe omitirse o ser `null` al crear.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub metadata: Option<serde_json::Value>,
    pub parent_id: Option<i32>,
    /// Solo en `GET /tasks` (árbol anidado).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TaskDto>,
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: i32,
    user_id: i32,
    title: String,
    description: Option<String>,
    completed: bool,
    metadata: serde_json::Value,
    parent_id: Option<i32>,
}

const TASK_SELECT: &str =
    "SELECT id, user_id, title, description, completed, metadata, parent_id FROM tasks";

impl TaskRow {
    fn into_dto(self) -> TaskDto {
        TaskDto {
            id: Some(self.id),
            title: Some(self.title),
            description: self.description,
            completed: Some(self.completed),
            metadata: Some(self.metadata),
            parent_id: self.parent_id,
            children: Vec::new(),
        }
    }
}

/// Convierte filas planas en bosque de [`TaskDto`] con `children`.
fn build_task_tree(tasks: Vec<TaskRow>) -> Vec<TaskDto> {
    use std::collections::HashSet;

    let ids: HashSet<i32> = tasks.iter().map(|task| task.id).collect();
    let mut by_parent: HashMap<Option<i32>, Vec<&TaskRow>> = HashMap::new();

    for task in &tasks {
        let parent_key = match task.parent_id {
            Some(parent_id) if ids.contains(&parent_id) => Some(parent_id),
            _ => None,
        };
        by_parent.entry(parent_key).or_default().push(task);
    }

    fn build_subtree(
        parent_id: Option<i32>,
        by_parent: &HashMap<Option<i32>, Vec<&TaskRow>>,
    ) -> Vec<TaskDto> {
        let mut siblings: Vec<&TaskRow> = by_parent.get(&parent_id).cloned().unwrap_or_default();
        siblings.sort_by_key(|task| task.id);

        siblings
            .into_iter()
            .map(|task| TaskDto {
                id: Some(task.id),
                title: Some(task.title.clone()),
                description: task.description.clone(),
                completed: Some(task.completed),
                metadata: Some(task.metadata.clone()),
                parent_id: task.parent_id,
                children: build_subtree(Some(task.id), by_parent),
            })
            .collect()
    }

    let mut roots = build_subtree(None, &by_parent);
    roots.sort_by_key(|task| std::cmp::Reverse(task.id.unwrap_or(0)));
    roots
}

async fn fetch_task_row(pool: &PgPool, id: i32, auth: &AuthUser) -> Result<TaskRow, StatusCode> {
    let task = sqlx::query_as::<_, TaskRow>(&format!("{TASK_SELECT} WHERE id = $1"))
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
) -> Result<Json<Vec<TaskDto>>, StatusCode> {
    let tasks = if auth.is_admin {
        sqlx::query_as::<_, TaskRow>(&format!("{TASK_SELECT} ORDER BY id ASC"))
            .fetch_all(&state.pool)
            .await
    } else {
        sqlx::query_as::<_, TaskRow>(&format!("{TASK_SELECT} WHERE user_id = $1 ORDER BY id ASC"))
            .bind(auth.id)
            .fetch_all(&state.pool)
            .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(build_task_tree(tasks)))
}

/// `POST /tasks` — crea tarea o subtarea (`parent_id` opcional en el DTO).
pub async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<TaskDto>,
) -> Result<Json<TaskDto>, StatusCode> {
    if payload.id.is_some() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let title = payload
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let user_id = auth.id;
    if let Some(parent_id) = payload.parent_id {
        validate_parent_belongs_to_user(&state.pool, parent_id, user_id).await?;
    }

    let metadata = payload.metadata.unwrap_or_else(|| serde_json::json!({}));
    let task = sqlx::query_as::<_, TaskRow>(
        "INSERT INTO tasks (user_id, title, description, metadata, parent_id)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, user_id, title, description, completed, metadata, parent_id",
    )
    .bind(user_id)
    .bind(title)
    .bind(payload.description)
    .bind(metadata)
    .bind(payload.parent_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task.into_dto()))
}

/// `PUT /tasks/:id` — reemplaza los campos enviados en el DTO.
pub async fn update_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<TaskDto>,
) -> Result<Json<TaskDto>, StatusCode> {
    let title = payload
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let completed = payload.completed.ok_or(StatusCode::BAD_REQUEST)?;
    let metadata = payload.metadata.ok_or(StatusCode::BAD_REQUEST)?;

    validate_parent_id(id, payload.parent_id)?;
    let existing = fetch_task_row(&state.pool, id, &auth).await?;

    if let Some(parent_id) = payload.parent_id {
        validate_parent_belongs_to_user(&state.pool, parent_id, existing.user_id).await?;
    }

    let task = sqlx::query_as::<_, TaskRow>(
        "UPDATE tasks SET title = $1, description = $2, completed = $3, metadata = $4, parent_id = $5
         WHERE id = $6
         RETURNING id, user_id, title, description, completed, metadata, parent_id",
    )
    .bind(title)
    .bind(payload.description)
    .bind(completed)
    .bind(metadata)
    .bind(payload.parent_id)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task.into_dto()))
}

/// `PATCH /tasks/:id/description` — actualiza solo `description` del DTO.
pub async fn patch_description(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<TaskDto>,
) -> Result<Json<TaskDto>, StatusCode> {
    fetch_task_row(&state.pool, id, &auth).await?;

    let task = sqlx::query_as::<_, TaskRow>(
        "UPDATE tasks SET description = $1
         WHERE id = $2
         RETURNING id, user_id, title, description, completed, metadata, parent_id",
    )
    .bind(payload.description)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task.into_dto()))
}

/// `PATCH /tasks/:id/metadata` — actualiza solo `metadata` del DTO.
pub async fn patch_metadata(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<TaskDto>,
) -> Result<Json<TaskDto>, StatusCode> {
    let metadata = payload.metadata.ok_or(StatusCode::BAD_REQUEST)?;
    fetch_task_row(&state.pool, id, &auth).await?;

    let task = sqlx::query_as::<_, TaskRow>(
        "UPDATE tasks SET metadata = $1
         WHERE id = $2
         RETURNING id, user_id, title, description, completed, metadata, parent_id",
    )
    .bind(metadata)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(task.into_dto()))
}

/// `POST /tasks/:id/toggle` — invierte el campo `completed`.
pub async fn toggle_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<bool>, StatusCode> {
    fetch_task_row(&state.pool, id, &auth).await?;

    sqlx::query("UPDATE tasks SET completed = NOT completed WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i32, parent_id: Option<i32>, title: &str) -> TaskRow {
        TaskRow {
            id,
            user_id: 1,
            title: title.to_string(),
            description: None,
            completed: false,
            metadata: serde_json::json!({}),
            parent_id,
        }
    }

    #[test]
    fn build_task_tree_nests_by_parent_id() {
        let tree = build_task_tree(vec![
            row(1, None, "root"),
            row(2, Some(1), "child"),
            row(3, None, "other-root"),
        ]);

        assert_eq!(tree.len(), 2);
        let root = tree.iter().find(|t| t.id == Some(1)).unwrap();
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].id, Some(2));
    }

    #[test]
    fn build_task_tree_orphan_parent_becomes_root() {
        let tree = build_task_tree(vec![row(10, Some(999), "orphan")]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].id, Some(10));
        assert!(tree[0].children.is_empty());
    }

    #[test]
    fn validate_parent_id_rejects_self_reference() {
        assert_eq!(validate_parent_id(5, Some(5)), Err(StatusCode::BAD_REQUEST));
        assert!(validate_parent_id(5, Some(3)).is_ok());
        assert!(validate_parent_id(5, None).is_ok());
    }
}
