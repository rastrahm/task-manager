use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::rc::Rc;

pub const API_URL: &str = "http://localhost:5040/tasks";

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

pub async fn fetch_tasks(client: &Rc<reqwest::Client>) -> Result<Vec<Task>, String> {
    let response = client
        .get(API_URL)
        .send()
        .await
        .map_err(|e| format!("No se pudo conectar con el backend: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "El servidor respondió con un error: {}",
            response.status()
        ));
    }

    response
        .json::<Vec<Task>>()
        .await
        .map_err(|e| format!("No se pudieron interpretar las tareas: {e}"))
}

pub async fn create_task_full(
    client: &Rc<reqwest::Client>,
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
    client
        .post(API_URL)
        .json(&new_task)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Task>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn update_task(
    client: &Rc<reqwest::Client>,
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
    client
        .put(format!("{API_URL}/{id}"))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Task>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn toggle_task(
    client: &Rc<reqwest::Client>,
    id: i32,
) -> Result<bool, String> {
    client
        .post(format!("{API_URL}/{id}/toggle"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<bool>()
        .await
        .map_err(|e| e.to_string())
}
