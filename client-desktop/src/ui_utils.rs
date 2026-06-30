use gtk4::prelude::*;
use gtk4::Application;
use std::cell::RefCell;

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

pub fn show_confirm_dialog(
    app: &Application,
    title: &str,
    message: &str,
    on_confirm: impl FnOnce() + 'static,
) {
    let dialog = gtk4::MessageDialog::builder()
        .modal(true)
        .buttons(gtk4::ButtonsType::YesNo)
        .text(title)
        .secondary_text(message)
        .build();

    if let Some(window) = app.active_window() {
        dialog.set_transient_for(Some(&window));
    }

    let on_confirm = RefCell::new(Some(on_confirm));

    dialog.connect_response(move |dialog, response| {
        if response == gtk4::ResponseType::Yes {
            if let Some(callback) = on_confirm.borrow_mut().take() {
                callback();
            }
        }
        dialog.close();
    });

    dialog.present();
}
