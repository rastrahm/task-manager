//! Runtime Tokio en segundo plano para reqwest/hyper desde GTK.

use std::future::Future;
use std::sync::OnceLock;
use tokio::runtime::{Handle, Runtime};

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Arranca el runtime (idempotente). Llamar al inicio de `main`.
pub fn init() {
    RUNTIME.get_or_init(|| {
        let runtime = Runtime::new().expect("No se pudo crear el runtime Tokio");
        let keep_alive = runtime.handle().clone();
        std::thread::spawn(move || {
            keep_alive.block_on(std::future::pending::<()>());
        });
        runtime
    });
}

fn handle() -> Handle {
    RUNTIME
        .get()
        .expect("runtime::init() debe llamarse antes de peticiones HTTP")
        .handle()
        .clone()
}

/// Ejecuta una future de red en Tokio y devuelve el resultado al hilo de GTK.
pub async fn run<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    handle().spawn(future).await.expect("tarea Tokio cancelada")
}
