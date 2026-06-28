mod dbus_service;
mod task_api;
mod ui_utils;

use gtk4::{
    prelude::*,
    Application, Box, Button, Entry, ListBox, Orientation, ScrolledWindow,
};
use std::cell::RefCell;
use std::rc::Rc;
use glib::clone;

const APP_ID: &str = "com.rolando.TaskManagerDesktop";

#[tokio::main]
async fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .default_width(400)
        .default_height(600)
        .build();

    let client = Rc::new(reqwest::Client::new());
    let tasks_data: Rc<RefCell<Vec<task_api::Task>>> = Rc::new(RefCell::new(Vec::new()));

    let header_bar = gtk4::HeaderBar::builder()
        .title_widget(&gtk4::Label::builder().label("Gestor de Tareas").build())
        .build();
    window.set_titlebar(Some(&header_bar));

    let refresh_button = Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Actualizar tareas")
        .build();
    header_bar.pack_end(&refresh_button);

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .build();

    let input_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .build();
    main_box.append(&input_box);

    let new_task_entry = Entry::builder()
        .placeholder_text("Nueva tarea...")
        .hexpand(true)
        .build();
    input_box.append(&new_task_entry);

    let add_button = Button::builder().label("Agregar").build();
    input_box.append(&add_button);

    let tasks_list_box = ListBox::builder()
        .css_classes(["task-list"])
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

    glib::spawn_future_local(clone!(@strong client, @strong app, @strong tasks_list_box, @strong tasks_data => async move {
        task_api::fetch_tasks_request(&client, &app, &tasks_list_box, &tasks_data).await;
    }));

    refresh_button.connect_clicked(clone!(@strong client, @strong app, @strong tasks_list_box, @strong tasks_data => move |_| {
        glib::spawn_future_local(clone!(@strong client, @strong app, @strong tasks_list_box, @strong tasks_data => async move {
            task_api::fetch_tasks_request(&client, &app, &tasks_list_box, &tasks_data).await;
        }));
    }));

    add_button.connect_clicked(clone!(@strong client, @strong app, @strong new_task_entry, @strong tasks_list_box, @strong tasks_data => move |_| {
        let title = new_task_entry.text().to_string();
        if title.is_empty() {
            ui_utils::show_error_dialog(&app, "Advertencia", "El título no puede estar vacío.");
            return;
        }
        new_task_entry.set_text("");

        glib::spawn_future_local(clone!(@strong client, @strong app, @strong tasks_list_box, @strong tasks_data => async move {
            if task_api::create_task_request(&client, title, None).await.is_ok() {
                task_api::fetch_tasks_request(&client, &app, &tasks_list_box, &tasks_data).await;
            } else {
                ui_utils::show_error_dialog(&app, "Error", "No se pudo crear la tarea.");
            }
        }));
    }));

    new_task_entry.connect_activate(clone!(@strong add_button => move |_| {
        add_button.emit_clicked();
    }));
}
