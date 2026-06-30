//! Metadatos opcionales de una tarea (`metadata` JSON del backend).
//!
//! Campos admitidos: prioridad, fecha límite y etiquetas. Se usan en el
//! formulario de tareas y en el resumen mostrado en la lista.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Valores válidos de prioridad en el selector del formulario.
pub const PRIORITIES: &[&str] = &["baja", "media", "alta"];

/// Subconjunto tipado del campo `metadata` de una tarea.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl TaskMetadata {
    /// Parsea `metadata` JSON; devuelve valores por defecto si el JSON no encaja.
    pub fn from_value(value: &Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    /// Serializa a `serde_json::Value` para enviar al API.
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| Value::Object(Default::default()))
    }

    /// Texto compacto para mostrar bajo el título en la lista (ej. `[alta] · 2026-03-20 · #casa`).
    pub fn summary_label(&self) -> Option<String> {
        let mut parts = Vec::new();

        if let Some(priority) = &self.priority {
            parts.push(format!("[{priority}]"));
        }
        if let Some(date) = &self.due_date {
            parts.push(date.clone());
        }
        if let Some(tags) = &self.tags {
            if !tags.is_empty() {
                parts.push(
                    tags.iter()
                        .map(|t| format!("#{t}"))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" · "))
        }
    }
}

/// Convierte texto del campo etiquetas (`"a, b"`) a lista; `None` si queda vacío.
pub fn parse_tags_input(input: &str) -> Option<Vec<String>> {
    let tags: Vec<String> = input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    if tags.is_empty() {
        None
    } else {
        Some(tags)
    }
}

/// Formatea etiquetas para rellenar el campo de texto del formulario.
pub fn tags_to_input(tags: &Option<Vec<String>>) -> String {
    tags.as_ref()
        .map(|list| list.join(", "))
        .unwrap_or_default()
}
