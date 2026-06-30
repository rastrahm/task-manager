//! Diálogo de inicio de sesión (ventana de aplicación, compatible con Unity/GTK).
//!
//! Presenta usuario y contraseña; al autenticar correctamente invoca el callback
//! con `true`. Si el usuario cierra la ventana o pulsa Salir, se invoca con `false`.

use crate::api_client::ApiClient;
use crate::runtime;
use crate::ui_utils::show_error_dialog;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Application, Box, Button, Entry, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Muestra el login y ejecuta el callback una sola vez al terminar.
pub fn show_login_dialog(
    app: &Application,
    api: &Arc<ApiClient>,
    on_complete: impl FnOnce(bool) + 'static,
) {
    let on_complete = Rc::new(RefCell::new(Some(on_complete)));
    let finish = {
        let on_complete = on_complete.clone();
        Rc::new(move |success: bool| {
            if let Some(callback) = on_complete.borrow_mut().take() {
                callback(success);
            }
        })
    };

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Iniciar sesión")
        .default_width(400)
        .default_height(300)
        .resizable(false)
        .build();

    let root = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .build();

    root.append(
        &Label::builder()
            .label("Gestor del Día a Día")
            .css_classes(["title-2"])
            .build(),
    );

    root.append(&form_label("Usuario"));
    let username_entry = Entry::builder()
        .placeholder_text("Nombre de usuario")
        .hexpand(true)
        .build();
    root.append(&username_entry);

    root.append(&form_label("Contraseña"));
    let password_entry = Entry::builder()
        .placeholder_text("Contraseña")
        .visibility(false)
        .hexpand(true)
        .build();
    root.append(&password_entry);

    let status_label = Label::builder()
        .label("")
        .css_classes(["dim-label"])
        .xalign(0.0)
        .build();
    root.append(&status_label);

    let button_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::End)
        .build();

    let cancel_button = Button::builder().label("Salir").build();
    let login_button = Button::builder()
        .label("Entrar")
        .css_classes(["suggested-action"])
        .build();
    button_box.append(&cancel_button);
    button_box.append(&login_button);
    root.append(&button_box);

    window.set_child(Some(&root));

    let finish_cancel = finish.clone();
    cancel_button.connect_clicked(clone!(@strong window, @strong finish_cancel => move |_| {
        window.close();
        finish_cancel(false);
    }));

    window.connect_close_request(clone!(@strong finish, @strong on_complete => move |_| {
        if on_complete.borrow().is_some() {
            finish(false);
        }
        gtk4::glib::Propagation::Proceed
    }));

    let app_login = app.clone();
    let api_login = api.clone();
    let login_button_async = login_button.clone();
    let finish_login = finish.clone();
    login_button.connect_clicked(
        clone!(@strong window, @strong username_entry, @strong password_entry, @strong login_button_async, @strong finish_login, @strong status_label => move |_| {
            let username = username_entry.text().to_string();
            let password = password_entry.text().to_string();
            if username.trim().is_empty() || password.is_empty() {
                show_error_dialog(&app_login, "Advertencia", "Usuario y contraseña son obligatorios.");
                return;
            }

            login_button_async.set_sensitive(false);
            status_label.set_label("Conectando con el servidor…");
            let app_async = app_login.clone();
            let api_async = api_login.clone();
            let login_button_restore = login_button_async.clone();
            let finish_async = finish_login.clone();
            let window_async = window.clone();
            let status_restore = status_label.clone();
            let username = username.trim().to_string();

            glib::spawn_future_local(async move {
                let api = Arc::clone(&api_async);
                let result = runtime::run(async move { api.login(&username, &password).await }).await;

                match result {
                    Ok(()) => {
                        // Completar antes de cerrar: close_request llamaría finish(false).
                        finish_async(true);
                        window_async.close();
                    }
                    Err(message) => {
                        status_restore.set_label("");
                        show_error_dialog(&app_async, "Error de autenticación", &message);
                        login_button_restore.set_sensitive(true);
                    }
                }
            });
        }),
    );

    password_entry.connect_activate(clone!(@strong login_button => move |_| {
        login_button.emit_clicked();
    }));

    window.present();
}

fn form_label(text: &str) -> Label {
    Label::builder().label(text).xalign(0.0).build()
}
