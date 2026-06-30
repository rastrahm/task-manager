use crate::task_api::Task;
use crate::task_form::{open_task_form, TaskFormMode};
use crate::task_metadata::TaskMetadata;
use crate::task_api;
use crate::ui_utils::show_error_dialog;
use glib::{clone, ControlFlow};
use gtk4::prelude::*;
use gtk4::{Align, Application, Box, Button, CheckButton, Label, ListBox, ListBoxRow, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct TaskListContext {
    pub client: Rc<reqwest::Client>,
    pub app: Application,
    pub tasks_list_box: ListBox,
    pub tasks_data: Rc<RefCell<Vec<Task>>>,
}

fn append_task_rows(ctx: &TaskListContext, tasks: &[Task], depth: usize) {
    for task in tasks {
        let row_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(10 + (depth as i32 * 24))
            .margin_end(10)
            .build();

        let check_button = CheckButton::builder()
            .active(task.completed)
            .valign(Align::Start)
            .build();
        row_box.append(&check_button);

        let content_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .build();

        let title_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let task_label = Label::builder()
            .label(&task.title)
            .halign(Align::Start)
            .hexpand(true)
            .xalign(0.0)
            .build();
        if task.completed {
            task_label.add_css_class("completed-task");
        }
        title_row.append(&task_label);

        let edit_button = Button::builder()
            .icon_name("document-edit-symbolic")
            .tooltip_text("Editar tarea")
            .build();
        title_row.append(&edit_button);

        let add_subtask_button = Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Agregar subtarea")
            .build();
        title_row.append(&add_subtask_button);

        content_box.append(&title_row);

        if let Some(desc) = &task.description {
            if !desc.trim().is_empty() {
                let desc_label = Label::builder()
                    .label(desc)
                    .halign(Align::Start)
                    .xalign(0.0)
                    .css_classes(["dim-label"])
                    .wrap(true)
                    .build();
                if task.completed {
                    desc_label.add_css_class("completed-task");
                }
                content_box.append(&desc_label);
            }
        }

        let meta = TaskMetadata::from_value(&task.metadata);
        if let Some(summary) = meta.summary_label() {
            content_box.append(
                &Label::builder()
                    .label(&summary)
                    .halign(Align::Start)
                    .xalign(0.0)
                    .css_classes(["caption"])
                    .wrap(true)
                    .build(),
            );
        }

        row_box.append(&content_box);

        let row = ListBoxRow::builder().build();
        row.set_child(Some(&row_box));
        ctx.tasks_list_box.append(&row);

        let check_button_clone = check_button.clone();
        let task_clone = task.clone();
        let ctx_toggle = ctx.clone();

        check_button.connect_toggled(clone!(@strong task_label, @strong ctx_toggle, @strong check_button_clone => move |btn| {
            let current_task_id = task_clone.id;
            if btn.is_active() {
                task_label.add_css_class("completed-task");
            } else {
                task_label.remove_css_class("completed-task");
            }

            glib::spawn_future_local(clone!(@strong ctx_toggle, @strong check_button_clone => async move {
                if let Err(e) = task_api::toggle_task(&ctx_toggle.client, current_task_id).await {
                    show_error_dialog(
                        &ctx_toggle.app,
                        "Error al alternar tarea",
                        &format!("No se pudo alternar el estado de la tarea {current_task_id}: {e}"),
                    );
                    glib::idle_add_local(clone!(@strong check_button_clone => move || {
                        check_button_clone.set_active(!check_button_clone.is_active());
                        ControlFlow::Break
                    }));
                } else {
                    refresh_task_list(&ctx_toggle).await;
                }
            }));
        }));

        let ctx_edit = ctx.clone();
        let task_for_edit = task.clone();
        edit_button.connect_clicked(clone!(@strong ctx_edit => move |_| {
            let ctx = ctx_edit.clone();
            open_task_form(
                &ctx.app,
                &ctx.client,
                &ctx,
                TaskFormMode::Edit(task_for_edit.clone()),
            );
        }));

        let ctx_subtask = ctx.clone();
        let parent_id = task.id;
        add_subtask_button.connect_clicked(clone!(@strong ctx_subtask => move |_| {
            let ctx = ctx_subtask.clone();
            open_task_form(
                &ctx.app,
                &ctx.client,
                &ctx,
                TaskFormMode::Create {
                    parent_id: Some(parent_id),
                },
            );
        }));

        if !task.children.is_empty() {
            append_task_rows(ctx, &task.children, depth + 1);
        }
    }
}

fn render_tasks(ctx: &TaskListContext, tasks: Vec<Task>) {
    while let Some(child) = ctx.tasks_list_box.first_child() {
        ctx.tasks_list_box.remove(&child);
    }
    ctx.tasks_data.replace(tasks.clone());
    append_task_rows(ctx, &tasks, 0);
}

pub async fn refresh_task_list(ctx: &TaskListContext) {
    match task_api::fetch_tasks(&ctx.client).await {
        Ok(tasks) => {
            let ctx = ctx.clone();
            glib::idle_add_local(move || {
                render_tasks(&ctx, tasks.clone());
                ControlFlow::Break
            });
        }
        Err(e) => show_error_dialog(&ctx.app, "Error", &e),
    }
}

pub async fn load_tasks(ctx: &TaskListContext) {
    refresh_task_list(ctx).await;
}
