use gtk::prelude::*;
use gtk::{self, glib, Application, ApplicationWindow, Button, Orientation};

const APP_ID: &str = "smart_psu";

pub fn client_ui() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(popup);

    app.run()
}

fn popup(app: &Application) {
    let button_sleep = Button::builder()
        .label("Sleep")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let button_hibernate = Button::builder()
        .label("Hibernate")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let button_shutdown = Button::builder()
        .label("Shutdown")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let button_exit = Button::builder()
        .label("Exit")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .focus_on_click(true)
        .build();

    let button_width = 120;
    let button_height = 10;
    button_sleep.set_size_request(button_width, button_height);
    button_hibernate.set_size_request(button_width, button_height);
    button_shutdown.set_size_request(button_width, button_height);
    button_exit.set_size_request(button_width, button_height);

    let gtk_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(100)
        .margin_bottom(100)
        .margin_start(100)
        .margin_end(100)
        .build();

    gtk_box.append(&button_sleep);
    gtk_box.append(&button_hibernate);
    gtk_box.append(&button_shutdown);
    gtk_box.append(&button_exit);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Power is out")
        .child(&gtk_box)
        .build();

    window.fullscreen();
    window.present();
}
