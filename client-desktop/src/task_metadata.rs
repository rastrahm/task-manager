use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PRIORITIES: &[&str] = &["baja", "media", "alta"];

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
    pub fn from_value(value: &Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| Value::Object(Default::default()))
    }

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

pub fn tags_to_input(tags: &Option<Vec<String>>) -> String {
    tags.as_ref()
        .map(|list| list.join(", "))
        .unwrap_or_default()
}
