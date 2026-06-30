use crate::task_api::{self, Task};
use crate::task_list::TaskListContext;
use crate::task_metadata::{parse_tags_input, tags_to_input, TaskMetadata, PRIORITIES};
use crate::task_list;
use crate::ui_utils::show_error_dialog;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, DropDown, Entry, Label, Orientation, ScrolledWindow, StringList,
    TextView, Window,
};
use std::rc::Rc;

pub enum TaskFormMode {
    Create { parent_id: Option<i32> },
    Edit(Task),
}

struct TaskFormResult {
    title: String,
    description: Option<String>,
    metadata: TaskMetadata,
}

pub fn open_task_form(
    app: &Application,
    client: &Rc<reqwest::Client>,
    ctx: &TaskListContext,
    mode: TaskFormMode,
) {
    let is_edit = matches!(mode, TaskFormMode::Edit(_));
    let window_title = if is_edit {
        "Editar tarea"
    } else if matches!(mode, TaskFormMode::Create { parent_id: Some(_) }) {
        "Nueva subtarea"
    } else {
        "Nueva tarea"
    };

    let window = Window::builder()
        .title(window_title)
        .modal(true)
        .default_width(520)
        .default_height(480)
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

    if let TaskFormMode::Create {
        parent_id: Some(pid),
    } = &mode
    {
        root.append(
            &Label::builder()
                .label(&format!("Subtarea de la tarea #{pid}"))
                .xalign(0.0)
                .build(),
        );
    }

    root.append(&form_label("Título *"));
    let title_entry = Entry::builder()
        .placeholder_text("Título de la tarea")
        .hexpand(true)
        .build();
    root.append(&title_entry);

    root.append(&form_label("Descripción"));
    let description_view = TextView::builder()
        .wrap_mode(gtk4::WrapMode::Word)
        .height_request(100)
        .build();
    let description_scroll = ScrolledWindow::builder()
        .min_content_height(100)
        .child(&description_view)
        .build();
    root.append(&description_scroll);

    root.append(&form_label("Prioridad"));
    let priority_model = StringList::new(&["", "baja", "media", "alta"]);
    let priority_drop = DropDown::builder().model(&priority_model).build();
    root.append(&priority_drop);

    root.append(&form_label("Fecha límite (AAAA-MM-DD)"));
    let due_date_entry = Entry::builder()
        .placeholder_text("2026-03-20")
        .build();
    root.append(&due_date_entry);

    root.append(&form_label("Etiquetas (separadas por coma)"));
    let tags_entry = Entry::builder()
        .placeholder_text("casa, urgente, trabajo")
        .build();
    root.append(&tags_entry);

    let button_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::End)
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

    if let TaskFormMode::Edit(task) = &mode {
        title_entry.set_text(&task.title);
        if let Some(desc) = &task.description {
            description_view.buffer().set_text(desc);
        }
        let meta = TaskMetadata::from_value(&task.metadata);
        if let Some(priority) = &meta.priority {
            if let Some(idx) = PRIORITIES.iter().position(|p| *p == priority) {
                priority_drop.set_selected((idx + 1) as u32);
            }
        }
        if let Some(date) = &meta.due_date {
            due_date_entry.set_text(date);
        }
        tags_entry.set_text(&tags_to_input(&meta.tags));
    }

    cancel_button.connect_clicked(clone!(@strong window => move |_| {
        window.close();
    }));

    let app_save = app.clone();
    let client_save = client.clone();
    let ctx_save = ctx.clone();
    let mode_save = mode;

    save_button.connect_clicked(clone!(@strong window, @strong title_entry, @strong description_view, @strong priority_drop, @strong due_date_entry, @strong tags_entry => move |_| {
        let title = title_entry.text().to_string();
        if title.trim().is_empty() {
            show_error_dialog(&app_save, "Advertencia", "El título no puede estar vacío.");
            return;
        }

        let description_buffer = description_view.buffer();
        let (start, end) = description_buffer.bounds();
        let description_text = description_buffer.text(&start, &end, true).to_string();
        let description = if description_text.trim().is_empty() {
            None
        } else {
            Some(description_text)
        };

        let selected = priority_drop.selected();
        let priority = if selected == 0 {
            None
        } else {
            PRIORITIES.get((selected - 1) as usize).map(|s| s.to_string())
        };

        let due_date_text = due_date_entry.text().to_string();
        let due_date = if due_date_text.trim().is_empty() {
            None
        } else {
            Some(due_date_text.trim().to_string())
        };

        let tags = parse_tags_input(&tags_entry.text());

        let form = TaskFormResult {
            title: title.trim().to_string(),
            description,
            metadata: TaskMetadata {
                priority,
                due_date,
                tags,
            },
        };

        let app = app_save.clone();
        let client = client_save.clone();
        let ctx = ctx_save.clone();
        let mode = match &mode_save {
            TaskFormMode::Create { parent_id } => TaskFormMode::Create { parent_id: *parent_id },
            TaskFormMode::Edit(task) => TaskFormMode::Edit(task.clone()),
        };

        glib::spawn_future_local(clone!(@strong window => async move {
            let result = match mode {
                TaskFormMode::Create { parent_id } => {
                    task_api::create_task_full(
                        &client,
                        form.title,
                        form.description,
                        form.metadata.to_value(),
                        parent_id,
                    )
                    .await
                }
                TaskFormMode::Edit(task) => {
                    task_api::update_task(
                        &client,
                        task.id,
                        form.title,
                        form.description,
                        task.completed,
                        form.metadata.to_value(),
                        task.parent_id,
                    )
                    .await
                }
            };

            match result {
                Ok(_) => {
                    task_list::refresh_task_list(&ctx).await;
                    window.close();
                }
                Err(e) => {
                    show_error_dialog(&app, "Error", &format!("No se pudo guardar la tarea: {e}"));
                }
            }
        }));
    }));

    window.present();
}

fn form_label(text: &str) -> Label {
    Label::builder().label(text).xalign(0.0).build()
}
