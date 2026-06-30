mod dbus_service;
mod task_api;
mod task_form;
mod task_list;
mod task_metadata;
mod ui_utils;

use gtk4::{prelude::*, Application, Box, Button, ListBox, Orientation, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;
use glib::clone;

use task_form::{open_task_form, TaskFormMode};
use task_list::{load_tasks, TaskListContext};

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
        .default_width(440)
        .default_height(640)
        .build();

    let client = Rc::new(reqwest::Client::new());
    let tasks_data: Rc<RefCell<Vec<task_api::Task>>> = Rc::new(RefCell::new(Vec::new()));
    let tasks_list_box = ListBox::builder()
        .css_classes(["task-list"])
        .build();

    let ctx = Rc::new(TaskListContext {
        client: client.clone(),
        app: app.clone(),
        tasks_list_box: tasks_list_box.clone(),
        tasks_data: tasks_data.clone(),
    });

    let header_bar = gtk4::HeaderBar::builder()
        .title_widget(&gtk4::Label::builder().label("Gestor de Tareas").build())
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
            &ctx.client,
            &ctx,
            TaskFormMode::Create { parent_id: None },
        );
    }));
}
