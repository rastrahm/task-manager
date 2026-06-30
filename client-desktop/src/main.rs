mod api_client;
mod config;
mod dbus_service;
mod login_dialog;
mod models;
mod runtime;
mod session_store;
mod task_api;
mod task_form;
mod task_list;
mod task_metadata;
mod ui_utils;
mod user_admin;

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Label, ListBox, Orientation, Popover, ScrolledWindow, Separator,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use glib::clone;

use api_client::ApiClient;
use login_dialog::show_login_dialog;
use task_form::{open_task_form, TaskFormMode};
use task_list::{load_tasks, TaskListContext};
use user_admin::open_user_admin_window;

const APP_ID: &str = "com.rolando.TaskManagerDesktop";

fn main() {
    config::init();
    runtime::init();

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        if app.active_window().is_some() {
            return;
        }
        present_initial_ui(app);
    });
    app.run();
}

fn present_initial_ui(app: &Application) {
    let api = Arc::new(ApiClient::new());

    if api.load_stored_session() {
        show_main_window(app, api.clone());
        let app = app.clone();
        glib::spawn_future_local(async move {
            let api_refresh = Arc::clone(&api);
            if runtime::run(async move { api_refresh.refresh_session_if_needed().await }).await.is_err() {
                api.clear_local_session();
                glib::idle_add_local(move || {
                    if let Some(window) = app.active_window() {
                        window.close();
                    }
                    let app_main = app.clone();
                    let api_main = api.clone();
                    show_login_dialog(&app, &api, move |success| {
                        if success {
                            show_main_window(&app_main, api_main);
                        } else {
                            app_main.quit();
                        }
                    });
                    glib::ControlFlow::Break
                });
            }
        });
        return;
    }

    let app_main = app.clone();
    let api_main = api.clone();
    show_login_dialog(app, &api, move |success| {
        if success {
            show_main_window(&app_main, api_main);
        } else {
            app_main.quit();
        }
    });
}

fn show_main_window(app: &Application, api: Arc<ApiClient>) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .default_width(440)
        .default_height(640)
        .build();

    let tasks_data: Rc<RefCell<Vec<task_api::Task>>> = Rc::new(RefCell::new(Vec::new()));
    let tasks_list_box = ListBox::builder()
        .css_classes(["task-list"])
        .build();

    let ctx = Rc::new(TaskListContext {
        api: api.clone(),
        app: app.clone(),
        tasks_list_box: tasks_list_box.clone(),
        tasks_data: tasks_data.clone(),
    });

    let username = api
        .username()
        .unwrap_or_else(|| "usuario".to_string());
    let is_admin = api.is_admin();

    let header_bar = gtk4::HeaderBar::builder()
        .title_widget(
            &Label::builder()
                .label("Gestor de Tareas")
                .build(),
        )
        .build();
    window.set_titlebar(Some(&header_bar));

    let new_task_button = Button::builder()
        .label("Nueva tarea")
        .tooltip_text("Crear tarea")
        .build();
    header_bar.pack_start(&new_task_button);

    let refresh_button = Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Actualizar tareas")
        .build();
    header_bar.pack_end(&refresh_button);

    let menu_button = Button::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Menú de usuario")
        .build();
    header_bar.pack_end(&menu_button);

    let menu_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .width_request(240)
        .build();

    menu_box.append(
        &Label::builder()
            .label(&format!("Sesión: {username}"))
            .xalign(0.0)
            .css_classes(["heading"])
            .build(),
    );

    if is_admin {
        menu_box.append(
            &Label::builder()
                .label("Administrador")
                .xalign(0.0)
                .css_classes(["caption", "success"])
                .build(),
        );
    }

    menu_box.append(&Separator::builder().build());

    let admin_users_button = Button::builder()
        .label("Administrar usuarios")
        .halign(gtk4::Align::Fill)
        .build();
    admin_users_button.set_visible(is_admin);
    menu_box.append(&admin_users_button);

    let logout_button = Button::builder()
        .label("Cerrar sesión")
        .halign(gtk4::Align::Fill)
        .build();
    menu_box.append(&logout_button);

    let popover = Popover::builder()
        .child(&menu_box)
        .autohide(true)
        .build();
    popover.set_parent(&menu_button);

    menu_button.connect_clicked(clone!(@strong popover => move |_| {
        popover.popup();
    }));

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    main_box.append(&tasks_list_box);

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .child(&main_box)
        .build();
    window.set_child(Some(&scrolled_window));

    let cmd_tx = dbus_service::setup_toggle_channel(&window);
    glib::spawn_future_local(async move {
        if let Err(e) = dbus_service::start_service(cmd_tx).await {
            eprintln!("Error al iniciar servicio D-Bus: {e}");
        }
    });

    window.present();

    glib::spawn_future_local(clone!(@strong ctx => async move {
        load_tasks(&ctx).await;
    }));

    refresh_button.connect_clicked(clone!(@strong ctx => move |_| {
        let ctx = ctx.clone();
        glib::spawn_future_local(async move {
            load_tasks(&ctx).await;
        });
    }));

    new_task_button.connect_clicked(clone!(@strong ctx => move |_| {
        let ctx = ctx.clone();
        open_task_form(
            &ctx.app,
            &ctx.api,
            &ctx,
            TaskFormMode::Create { parent_id: None },
        );
    }));

    let app_admin = app.clone();
    let api_admin = api.clone();
    admin_users_button.connect_clicked(move |_| {
        open_user_admin_window(&app_admin, &api_admin);
    });

    let app_logout = app.clone();
    let api_logout = api.clone();
    let window_logout = window.clone();
    logout_button.connect_clicked(move |_| {
        let app = app_logout.clone();
        let api = api_logout.clone();
        let window = window_logout.clone();
        glib::spawn_future_local(async move {
            let api_logout = Arc::clone(&api);
            let _ = runtime::run(async move { api_logout.logout().await }).await;
            glib::idle_add_local(move || {
                window.close();
                let app_main = app.clone();
                let api_main = api.clone();
                show_login_dialog(&app, &api, move |success| {
                    if success {
                        show_main_window(&app_main, api_main);
                    } else {
                        app_main.quit();
                    }
                });
                glib::ControlFlow::Break
            });
        });
    });
}
