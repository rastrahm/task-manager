//! Configuración del cliente (URL del API desde `.env` o variables de entorno).
//!
//! La URL base se resuelve una sola vez al arrancar mediante [`init`].

use std::sync::OnceLock;

static API_BASE: OnceLock<String> = OnceLock::new();

const DEFAULT_API_BASE_URL: &str = "http://localhost:5040";
const ENV_KEY: &str = "API_BASE_URL";

/// Carga `.env` del directorio de trabajo y deja lista la URL del API.
pub fn init() {
    let _ = dotenvy::dotenv();
    let _ = api_base_url();
}

/// URL base del backend, sin barra final (ej. `http://localhost:5040`).
pub fn api_base_url() -> &'static str {
    API_BASE.get_or_init(|| {
        std::env::var(ENV_KEY).unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_string())
    })
}
