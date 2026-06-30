//! Servicio D-Bus para mostrar u ocultar la ventana principal.
//!
//! Expone la interfaz `com.rolando.TaskManager.Window` en el bus de sesión.
//! El applet MATE u otros clientes pueden llamar a
//! [`TaskManagerDesktop::toggle_window`] sin acoplarse a GTK directamente.

use gtk4::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use zbus::interface;

/// Nombre del servicio en el bus de sesión D-Bus.
pub const BUS_NAME: &str = "com.rolando.TaskManager.Window";
/// Ruta del objeto que implementa la interfaz de ventana.
pub const OBJECT_PATH: &str = "/com/rolando/TaskManager/Window";

/// Conecta un canal que alterna visibilidad de la ventana en el hilo principal de GTK.
///
/// Debe llamarse antes de [`start_service`]; el receptor se consulta cada 50 ms.
pub fn setup_toggle_channel(window: &gtk4::ApplicationWindow) -> mpsc::Sender<()> {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));
    let window = window.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        while cmd_rx.lock().unwrap().try_recv().is_ok() {
            if window.is_visible() {
                window.hide();
            } else {
                window.present();
            }
        }
        glib::ControlFlow::Continue
    });

    cmd_tx
}

/// Implementación del objeto D-Bus que recibe comandos de toggle.
pub struct TaskManagerDesktop {
    cmd_tx: mpsc::Sender<()>,
}

impl TaskManagerDesktop {
    /// Crea el objeto con el emisor del canal configurado en [`setup_toggle_channel`].
    pub fn new(cmd_tx: mpsc::Sender<()>) -> Self {
        Self { cmd_tx }
    }
}

#[interface(name = "com.rolando.TaskManager.Window")]
impl TaskManagerDesktop {
    /// Muestra la ventana si estaba oculta, o la oculta si estaba visible.
    async fn toggle_window(&mut self) -> zbus::fdo::Result<()> {
        self.cmd_tx
            .send(())
            .map_err(|_| zbus::fdo::Error::Failed("No se pudo enviar comando a la ventana".into()))?;
        Ok(())
    }
}

/// Registra el servicio en el bus de sesión y se mantiene activo indefinidamente.
pub async fn start_service(cmd_tx: mpsc::Sender<()>) -> zbus::Result<()> {
    let state = TaskManagerDesktop::new(cmd_tx);
    let _connection = zbus::connection::Builder::session()?
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, state)?
        .build()
        .await?;

    std::future::pending::<()>().await;
    #[allow(unreachable_code)]
    Ok(())
}
