use gtk4::prelude::*;
use gtk4::{Application, Box, Dialog, Entry, Label, Orientation, ResponseType};
use glib::clone;
use std::cell::RefCell;
use std::rc::Rc;

pub fn show_error_dialog(app: &Application, title: &str, message: &str) {
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

pub fn show_subtask_dialog<F>(app: &Application, parent_id: i32, on_submit: F)
where
    F: FnOnce(String) + 'static,
{
    let dialog = Dialog::builder()
        .title("Nueva subtarea")
        .modal(true)
        .build();

    if let Some(window) = app.active_window() {
        dialog.set_transient_for(Some(&window));
    }

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    content.append(
        &Label::builder()
            .label(&format!("Subtarea de la tarea #{parent_id}"))
            .xalign(0.0)
            .build(),
    );

    let entry = Entry::builder()
        .placeholder_text("Título de la subtarea...")
        .activates_default(true)
        .build();
    content.append(&entry);

    dialog.content_area().append(&content);
    dialog.add_button("Cancelar", ResponseType::Cancel);
    dialog.add_button("Agregar", ResponseType::Accept);
    dialog.set_default_response(ResponseType::Accept);

    let on_submit = Rc::new(RefCell::new(Some(on_submit)));

    dialog.connect_response(clone!(@strong on_submit => move |dialog, response| {
        if response == ResponseType::Accept {
            let title = entry.text().to_string();
            if !title.trim().is_empty() {
                if let Some(callback) = on_submit.borrow_mut().take() {
                    callback(title);
                }
            }
        }
        dialog.close();
    }));

    dialog.present();
}
