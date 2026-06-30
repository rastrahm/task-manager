//! Ventana de administración de usuarios (solo `is_admin`).

use crate::api_client::ApiClient;
use crate::models::{CreateUserRequest, UpdateUserRequest, User};
use crate::runtime;
use crate::ui_utils::{show_confirm_dialog, show_error_dialog};
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, Box, Button, CheckButton, Entry, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, Window,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub fn open_user_admin_window(app: &Application, api: &Arc<ApiClient>) {
    if !api.is_admin() {
        show_error_dialog(
            app,
            "Acceso denegado",
            "Solo los administradores pueden gestionar usuarios.",
        );
        return;
    }

    let window = Window::builder()
        .title("Administración de usuarios")
        .default_width(560)
        .default_height(480)
        .build();

    if let Some(parent) = app.active_window() {
        window.set_transient_for(Some(&parent));
    }

    let users: Rc<RefCell<Vec<User>>> = Rc::new(RefCell::new(Vec::new()));
    let list_box = ListBox::builder().css_classes(["boxed-list"]).build();

    let root = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let toolbar = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let refresh_button = Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Actualizar lista")
        .build();
    let add_button = Button::builder()
        .label("Nuevo usuario")
        .css_classes(["suggested-action"])
        .build();
    let edit_button = Button::builder().label("Editar").build();
    let delete_button = Button::builder().label("Eliminar").build();

    toolbar.append(&refresh_button);
    toolbar.append(&add_button);
    toolbar.append(&edit_button);
    toolbar.append(&delete_button);
    root.append(&toolbar);

    let scroll = ScrolledWindow::builder()
        .vexpand(true)
        .child(&list_box)
        .build();
    root.append(&scroll);

    window.set_child(Some(&root));

    fn render_users(list_box: &ListBox, users: &[User]) {
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        for user in users {
            let row_box = Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(4)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(8)
                .margin_end(8)
                .build();

            let title = if user.is_admin {
                format!("{} (administrador)", user.username)
            } else {
                user.username.clone()
            };
            row_box.append(
                &Label::builder()
                    .label(&title)
                    .halign(Align::Start)
                    .xalign(0.0)
                    .build(),
            );

            let status = if user.is_active {
                "Activo"
            } else {
                "Inactivo"
            };
            row_box.append(
                &Label::builder()
                    .label(status)
                    .css_classes(["caption", "dim-label"])
                    .halign(Align::Start)
                    .xalign(0.0)
                    .build(),
            );

            let row = ListBoxRow::builder().build();
            row.set_child(Some(&row_box));
            list_box.append(&row);
        }
    }

    fn selected_user(list_box: &ListBox, users: &[User]) -> Option<User> {
        let idx = list_box.selected_row()?.index() as usize;
        users.get(idx).cloned()
    }

    async fn reload_users(
        app: &Application,
        api: &Arc<ApiClient>,
        list_box: &ListBox,
        users: &Rc<RefCell<Vec<User>>>,
    ) {
        let api = Arc::clone(api);
        match runtime::run(async move { api.list_users().await }).await {
            Ok(fetched) => {
                users.replace(fetched.clone());
                let list_box = list_box.clone();
                glib::idle_add_local(move || {
                    render_users(&list_box, &fetched);
                    glib::ControlFlow::Break
                });
            }
            Err(message) => show_error_dialog(app, "Error", &message),
        }
    }

    let app_reload = app.clone();
    let api_reload = api.clone();
    let list_reload = list_box.clone();
    let users_reload = users.clone();
    glib::spawn_future_local(async move {
        reload_users(&app_reload, &api_reload, &list_reload, &users_reload).await;
    });

    refresh_button.connect_clicked(clone!(@strong app, @strong api, @strong list_box, @strong users => move |_| {
        let app = app.clone();
        let api = api.clone();
        let list_box = list_box.clone();
        let users = users.clone();
        glib::spawn_future_local(async move {
            reload_users(&app, &api, &list_box, &users).await;
        });
    }));

    add_button.connect_clicked(clone!(@strong app, @strong api, @strong list_box, @strong users => move |_| {
        let on_saved = Rc::new(clone!(@strong app, @strong api, @strong list_box, @strong users => move || {
            let app = app.clone();
            let api = api.clone();
            let list_box = list_box.clone();
            let users = users.clone();
            glib::spawn_future_local(async move {
                reload_users(&app, &api, &list_box, &users).await;
            });
        }));
        open_user_form(&app, &api, None, on_saved);
    }));

    edit_button.connect_clicked(clone!(@strong app, @strong api, @strong list_box, @strong users => move |_| {
        let Some(user) = selected_user(&list_box, &users.borrow()) else {
            show_error_dialog(&app, "Selección", "Elige un usuario de la lista.");
            return;
        };
        let on_saved = Rc::new(clone!(@strong app, @strong api, @strong list_box, @strong users => move || {
            let app = app.clone();
            let api = api.clone();
            let list_box = list_box.clone();
            let users = users.clone();
            glib::spawn_future_local(async move {
                reload_users(&app, &api, &list_box, &users).await;
            });
        }));
        open_user_form(&app, &api, Some(user), on_saved);
    }));

    delete_button.connect_clicked(clone!(@strong app, @strong api, @strong list_box, @strong users => move |_| {
        let Some(user) = selected_user(&list_box, &users.borrow()) else {
            show_error_dialog(&app, "Selección", "Elige un usuario de la lista.");
            return;
        };

        let app_confirm = app.clone();
        let api_confirm = api.clone();
        let list_confirm = list_box.clone();
        let users_confirm = users.clone();
        let username = user.username.clone();
        let user_id = user.id;

        show_confirm_dialog(
            &app,
            "Eliminar usuario",
            &format!("¿Eliminar al usuario «{username}»? Esta acción no se puede deshacer."),
            {
                let app_confirm = app_confirm.clone();
                let api_confirm = api_confirm.clone();
                let list_confirm = list_confirm.clone();
                let users_confirm = users_confirm.clone();
                move || {
                    glib::spawn_future_local(async move {
                        let api = Arc::clone(&api_confirm);
                        match runtime::run(async move { api.delete_user(user_id).await }).await {
                            Ok(()) => {
                                reload_users(
                                    &app_confirm,
                                    &api_confirm,
                                    &list_confirm,
                                    &users_confirm,
                                )
                                .await;
                            }
                            Err(message) => show_error_dialog(&app_confirm, "Error", &message),
                        }
                    });
                }
            },
        );
    }));

    window.present();
}

fn open_user_form(
    app: &Application,
    api: &Arc<ApiClient>,
    existing: Option<User>,
    on_saved: Rc<dyn Fn()>,
) {
    let is_edit = existing.is_some();
    let window = Window::builder()
        .title(if is_edit {
            "Editar usuario"
        } else {
            "Nuevo usuario"
        })
        .modal(true)
        .default_width(420)
        .default_height(360)
        .build();

    if let Some(parent) = app.active_window() {
        window.set_transient_for(Some(&parent));
    }

    let root = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    root.append(&form_label("Usuario *"));
    let username_entry = Entry::builder().hexpand(true).build();
    root.append(&username_entry);

    root.append(&form_label(if is_edit {
        "Nueva contraseña (dejar vacío para no cambiar)"
    } else {
        "Contraseña *"
    }));
    let password_entry = Entry::builder().visibility(false).hexpand(true).build();
    root.append(&password_entry);

    let admin_check = CheckButton::builder()
        .label("Administrador")
        .build();
    root.append(&admin_check);

    let active_check = CheckButton::builder()
        .label("Cuenta activa")
        .active(true)
        .build();
    if is_edit {
        root.append(&active_check);
    }

    let button_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::End)
        .build();
    let cancel_button = Button::builder().label("Cancelar").build();
    let save_button = Button::builder()
        .label(if is_edit { "Guardar" } else { "Crear" })
        .css_classes(["suggested-action"])
        .build();
    button_box.append(&cancel_button);
    button_box.append(&save_button);
    root.append(&button_box);

    window.set_child(Some(&root));

    if let Some(user) = &existing {
        username_entry.set_text(&user.username);
        admin_check.set_active(user.is_admin);
        active_check.set_active(user.is_active);
    }

    cancel_button.connect_clicked(clone!(@strong window => move |_| window.close()));

    let app_save = app.clone();
    let api_save = api.clone();
    let existing_id = existing.as_ref().map(|user| user.id);

    save_button.connect_clicked(clone!(@strong window, @strong username_entry, @strong password_entry, @strong admin_check, @strong active_check => move |_| {
        let username = username_entry.text().to_string();
        let password = password_entry.text().to_string();

        if username.trim().is_empty() {
            show_error_dialog(&app_save, "Advertencia", "El nombre de usuario es obligatorio.");
            return;
        }
        if !is_edit && password.is_empty() {
            show_error_dialog(&app_save, "Advertencia", "La contraseña es obligatoria.");
            return;
        }

        let is_admin_flag = admin_check.is_active();
        let is_active_flag = active_check.is_active();
        let app = app_save.clone();
        let api = api_save.clone();
        let on_saved = on_saved.clone();
        let window_async = window.clone();

        glib::spawn_future_local(async move {
            let api = Arc::clone(&api);
            let result = runtime::run(async move {
                if let Some(id) = existing_id {
                    api.update_user(
                        id,
                        &UpdateUserRequest {
                            username: Some(username.trim().to_string()),
                            password: if password.is_empty() {
                                None
                            } else {
                                Some(password)
                            },
                            is_admin: Some(is_admin_flag),
                            is_active: Some(is_active_flag),
                        },
                    )
                    .await
                    .map(|_| ())
                } else {
                    api.create_user(&CreateUserRequest {
                        username: username.trim().to_string(),
                        password,
                        is_admin: is_admin_flag,
                    })
                    .await
                    .map(|_| ())
                }
            })
            .await;

            match result {
                Ok(()) => {
                    on_saved();
                    window_async.close();
                }
                Err(message) => show_error_dialog(&app, "Error", &message),
            }
        });
    }));

    window.present();
}

fn form_label(text: &str) -> Label {
    Label::builder().label(text).xalign(0.0).build()
}
