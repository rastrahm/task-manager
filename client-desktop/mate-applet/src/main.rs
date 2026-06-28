use gtk4::{prelude::*, Align, Application, ApplicationWindow, Box, Button, Orientation};
use glib::clone;
use std::process::Command;
use zbus::{fdo::DBusProxy, names::BusName, proxy, Connection};

const APPLET_ID: &str = "com.rolando.MateApplet";

const WINDOW_SERVICE: &str = "com.rolando.TaskManager.Window";

#[proxy(
    interface = "com.rolando.TaskManager.Window",
    default_service = "com.rolando.TaskManager.Window",
    default_path = "/com/rolando/TaskManager/Window"
)]
trait TaskManagerWindow {
    fn toggle_window(&self) -> zbus::Result<()>;
}

fn launch_desktop_app() {
    let desktop_bin = std::env::current_exe()
        .ok()
        .and_then(|exe| {
            exe.parent()?
                .parent()?
                .parent()?
                .join("../target/debug/client-desktop")
                .canonicalize()
                .ok()
        });

    if let Some(path) = desktop_bin {
        let _ = Command::new(path).spawn();
    } else {
        let _ = Command::new("client-desktop").spawn();
    }
}

async fn toggle_or_launch() -> Result<(), String> {
    let connection = Connection::session()
        .await
        .map_err(|e| format!("No se pudo conectar al bus D-Bus: {e}"))?;

    let dbus = DBusProxy::new(&connection)
        .await
        .map_err(|e| format!("No se pudo acceder a org.freedesktop.DBus: {e}"))?;

    let service_name = BusName::try_from(WINDOW_SERVICE)
        .map_err(|e| format!("Nombre de servicio inválido: {e}"))?;

    if !dbus.name_has_owner(service_name).await.unwrap_or(false) {
        launch_desktop_app();
        return Ok(());
    }

    let proxy = TaskManagerWindowProxy::new(&connection)
        .await
        .map_err(|e| format!("No se pudo conectar al servicio de ventana: {e}"))?;

    proxy
        .toggle_window()
        .await
        .map_err(|e| format!("No se pudo alternar la ventana: {e}"))
}

#[tokio::main]
async fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APPLET_ID).build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Task Manager Applet")
            .default_width(48)
            .default_height(48)
            .resizable(false)
            .decorated(false)
            .build();

        let button = Button::builder()
            .icon_name("view-list-symbolic")
            .tooltip_text("Abrir/Cerrar Gestor de Tareas")
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();
        container.append(&button);
        window.set_child(Some(&container));

        button.connect_clicked(clone!(@strong app => move |_| {
            let app = app.clone();
            glib::spawn_future_local(async move {
                if let Err(e) = toggle_or_launch().await {
                    eprintln!("Error: {e}");
                    glib::idle_add_local_once(move || {
                        show_error_dialog(&app, "Error", &e);
                    });
                }
            });
        }));

        window.present();
    });

    app.run()
}

fn show_error_dialog(app: &Application, title: &str, message: &str) {
    let dialog = gtk4::MessageDialog::builder()
        .modal(true)
        .buttons(gtk4::ButtonsType::Ok)
        .text(title)
        .secondary_text(message)
        .build();

    if let Some(window) = app.active_window() {
        dialog.set_transient_for(Some(&window));
    }

    dialog.present();
}
