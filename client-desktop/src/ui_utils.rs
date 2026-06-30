use gtk4::prelude::*;
use gtk4::Application;

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
