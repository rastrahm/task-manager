//! Persistencia local de la sesión (tokens + usuario, incl. `is_admin`).
//!
//! Guarda la sesión en `~/.config/task-manager/session.json` para restaurarla
//! al reiniciar la aplicación sin volver a pedir credenciales.

use crate::models::Session;
use std::fs;
use std::path::PathBuf;

fn session_path() -> Option<PathBuf> {
    let base = dirs::config_dir()?;
    Some(base.join("task-manager").join("session.json"))
}

/// Lee la sesión desde disco; devuelve `None` si no existe o el JSON es inválido.
pub fn load_session() -> Option<Session> {
    let path = session_path()?;
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Escribe la sesión en disco, creando el directorio si hace falta.
pub fn save_session(session: &Session) -> Result<(), String> {
    let path = session_path().ok_or("No se pudo resolver el directorio de configuración")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

/// Elimina el archivo de sesión local, si existe.
pub fn clear_session() {
    if let Some(path) = session_path() {
        let _ = fs::remove_file(path);
    }
}
