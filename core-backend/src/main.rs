//! Punto de entrada del binario `task-core`.
//!
//! La lógica y la documentación de la API viven en la biblioteca [`task_core`].
//! Genera la documentación con `cargo doc --open`.

#[tokio::main]
async fn main() {
    task_core::serve().await;
}
