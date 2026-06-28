use crate::ui_utils::{show_error_dialog, show_subtask_dialog};
use glib::{clone, ControlFlow};
use gtk4::prelude::*;
use gtk4::{Application, Box, Button, CheckButton, Label, ListBox, ListBoxRow, Orientation};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

pub const API_URL: &str = "http://localhost:5040/tasks";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub metadata: Value,
    #[serde(default)]
    pub parent_id: Option<i32>,
    #[serde(default)]
    pub children: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
    pub parent_id: Option<i32>,
}

#[derive(Clone)]
struct TaskListContext {
    client: Rc<reqwest::Client>,
    app: Application,
    tasks_list_box: ListBox,
    tasks_data: Rc<RefCell<Vec<Task>>>,
}

fn append_task_rows(ctx: &TaskListContext, tasks: &[Task], depth: usize) {
    for task in tasks {
        let row_content = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .margin_top(5)
            .margin_bottom(5)
            .margin_start(10 + (depth as i32 * 24))
            .build();

        let check_button = CheckButton::builder()
            .active(task.completed)
            .halign(gtk4::Align::Start)
            .build();
        row_content.append(&check_button);

        let task_label = Label::builder()
            .label(&task.title)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .xalign(0.0)
            .build();
        if task.completed {
            task_label.add_css_class("completed-task");
        }
        row_content.append(&task_label);

        let add_subtask_button = Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Agregar subtarea")
            .build();
        row_content.append(&add_subtask_button);

        let row = ListBoxRow::builder().build();
        row.set_child(Some(&row_content));
        ctx.tasks_list_box.append(&row);

        let check_button_clone = check_button.clone();
        let task_clone = task.clone();
        let ctx_for_toggle = ctx.clone();

        check_button.connect_toggled(clone!(@strong task_label, @strong ctx_for_toggle, @strong check_button_clone => move |btn| {
            let current_task_id = task_clone.id;
            if btn.is_active() {
                task_label.add_css_class("completed-task");
            } else {
                task_label.remove_css_class("completed-task");
            }

            glib::spawn_future_local(clone!(@strong ctx_for_toggle, @strong check_button_clone => async move {
                if let Err(e) = toggle_task_request(&ctx_for_toggle.client, current_task_id).await {
                    eprintln!("Error toggling task: {e:?}");
                    show_error_dialog(
                        &ctx_for_toggle.app,
                        "Error al alternar tarea",
                        &format!(
                            "No se pudo alternar el estado de la tarea {current_task_id}: {e}"
                        ),
                    );
                    glib::idle_add_local(clone!(@strong check_button_clone => move || {
                        check_button_clone.set_active(!check_button_clone.is_active());
                        ControlFlow::Break
                    }));
                } else {
                    fetch_tasks_request(
                        &ctx_for_toggle.client,
                        &ctx_for_toggle.app,
                        &ctx_for_toggle.tasks_list_box,
                        &ctx_for_toggle.tasks_data,
                    )
                    .await;
                }
            }));
        }));

        let ctx_for_subtask = ctx.clone();
        let parent_id = task.id;
        add_subtask_button.connect_clicked(clone!(@strong ctx_for_subtask => move |_| {
            let ctx = ctx_for_subtask.clone();
            let app = ctx.app.clone();
            show_subtask_dialog(&app, parent_id, move |title| {
                glib::spawn_future_local(async move {
                    match create_task_request(&ctx.client, title, Some(parent_id)).await {
                        Ok(_) => {
                            fetch_tasks_request(
                                &ctx.client,
                                &ctx.app,
                                &ctx.tasks_list_box,
                                &ctx.tasks_data,
                            )
                            .await;
                        }
                        Err(e) => {
                            show_error_dialog(
                                &ctx.app,
                                "Error",
                                &format!("No se pudo crear la subtarea: {e}"),
                            );
                        }
                    }
                });
            });
        }));

        if !task.children.is_empty() {
            append_task_rows(ctx, &task.children, depth + 1);
        }
    }
}

pub async fn fetch_tasks_request(
    client: &Rc<reqwest::Client>,
    app: &Application,
    tasks_list_box: &ListBox,
    tasks_data: &Rc<RefCell<Vec<Task>>>,
) {
    match client.get(API_URL).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Vec<Task>>().await {
                    Ok(tasks) => {
                        let tasks_for_ui = tasks;
                        let ctx = TaskListContext {
                            client: client.clone(),
                            app: app.clone(),
                            tasks_list_box: tasks_list_box.clone(),
                            tasks_data: tasks_data.clone(),
                        };

                        glib::idle_add_local(move || {
                            while let Some(child) = ctx.tasks_list_box.first_child() {
                                ctx.tasks_list_box.remove(&child);
                            }
                            ctx.tasks_data.replace(tasks_for_ui.clone());
                            append_task_rows(&ctx, &tasks_for_ui, 0);
                            ControlFlow::Break
                        });
                    }
                    Err(e) => {
                        show_error_dialog(
                            app,
                            "Error de datos",
                            &format!("No se pudieron interpretar las tareas: {e}"),
                        );
                    }
                }
            } else {
                show_error_dialog(
                    app,
                    "Error del Servidor",
                    &format!("El servidor respondió con un error: {}", response.status()),
                );
            }
        }
        Err(e) => {
            show_error_dialog(
                app,
                "Error de Conexión",
                &format!("No se pudo conectar con el backend: {e}"),
            );
        }
    }
}

pub async fn create_task_request(
    client: &Rc<reqwest::Client>,
    title: String,
    parent_id: Option<i32>,
) -> Result<Task, reqwest::Error> {
    let new_task = CreateTask {
        title,
        description: None,
        metadata: None,
        parent_id,
    };
    client
        .post(API_URL)
        .json(&new_task)
        .send()
        .await?
        .json::<Task>()
        .await
}

pub async fn toggle_task_request(
    client: &Rc<reqwest::Client>,
    id: i32,
) -> Result<bool, reqwest::Error> {
    client
        .post(format!("{API_URL}/{id}/toggle"))
        .send()
        .await?
        .json::<bool>()
        .await
}
