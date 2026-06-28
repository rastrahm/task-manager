use gtk4::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use zbus::interface;

pub const BUS_NAME: &str = "com.rolando.TaskManager.Window";
pub const OBJECT_PATH: &str = "/com/rolando/TaskManager/Window";

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

pub struct TaskManagerDesktop {
    cmd_tx: mpsc::Sender<()>,
}

impl TaskManagerDesktop {
    pub fn new(cmd_tx: mpsc::Sender<()>) -> Self {
        Self { cmd_tx }
    }
}

#[interface(name = "com.rolando.TaskManager.Window")]
impl TaskManagerDesktop {
    async fn toggle_window(&mut self) -> zbus::fdo::Result<()> {
        self.cmd_tx
            .send(())
            .map_err(|_| zbus::fdo::Error::Failed("No se pudo enviar comando a la ventana".into()))?;
        Ok(())
    }
}

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
