//! Acceso al API de tareas del backend.
//!
//! Define el modelo [`Task`] en formato árbol (`children`) y funciones para
//! listar, crear, actualizar y alternar el estado de completado.

use crate::api_client::ApiClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tarea tal como la devuelve `GET /tasks`: raíz o nodo con subtareas anidadas.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub metadata: Value,
    #[serde(default)]
    pub parent_id: Option<i32>,
    #[serde(default)]
    pub children: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateTask {
    title: String,
    description: Option<String>,
    metadata: Option<Value>,
    parent_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UpdateTask {
    title: String,
    description: Option<String>,
    completed: bool,
    metadata: Value,
    parent_id: Option<i32>,
}

/// Obtiene las tareas raíz con sus `children` anidados.
pub async fn fetch_tasks(api: &ApiClient) -> Result<Vec<Task>, String> {
    api.get("/tasks").await
}

/// Crea una tarea o subtarea con título, descripción, metadatos y padre opcional.
pub async fn create_task_full(
    api: &ApiClient,
    title: String,
    description: Option<String>,
    metadata: Value,
    parent_id: Option<i32>,
) -> Result<Task, String> {
    let new_task = CreateTask {
        title,
        description,
        metadata: Some(metadata),
        parent_id,
    };
    api.post("/tasks", &new_task).await
}

/// Reemplaza todos los campos editables de una tarea existente.
pub async fn update_task(
    api: &ApiClient,
    id: i32,
    title: String,
    description: Option<String>,
    completed: bool,
    metadata: Value,
    parent_id: Option<i32>,
) -> Result<Task, String> {
    let payload = UpdateTask {
        title,
        description,
        completed,
        metadata,
        parent_id,
    };
    api.put(&format!("/tasks/{id}"), &payload).await
}

/// Invierte el campo `completed` de la tarea indicada.
pub async fn toggle_task(api: &ApiClient, id: i32) -> Result<bool, String> {
    api.post_empty(&format!("/tasks/{id}/toggle")).await
}
