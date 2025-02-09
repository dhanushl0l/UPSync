use env_logger::{Builder, Env};
use glib::{clone, timeout_add_seconds};
use gtk::prelude::*;
use gtk::{self, glib, Application, ApplicationWindow, Button, Label, Orientation};
use std::env;
use upsync::core;

const APP_ID: &str = "com.dhanu.upsync";

fn main() {
    let env = Env::default().filter_or("LOG", "info");
    Builder::from_env(env).init();

    let default_action =
        core::get_default(&env::var("DEFAULT").unwrap_or_else(|_| "shutdown".to_string()));

    let sec = env::var("SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let message = format!("System will {} in {} seconds", default_action, sec);

    run_gui(message, sec);
}

fn run_gui(defaults: String, sec: u32) -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        popup(app, defaults.clone());

        timeout_add_seconds(sec, || {
            default();
            gtk::glib::ControlFlow::Break
        });
    });

    app.run()
}

fn popup(app: &Application, defaults: String) {
    let label = Label::builder().label(defaults).build();

    let button_sleep = Button::builder().label("Sleep").build();
    let button_hibernate = Button::builder().label("Hibernate").build();
    let button_shutdown = Button::builder().label("Shutdown").build();
    let button_ignore = Button::builder().label("ignore").build();

    let gtk_box_parent = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    gtk_box_parent.append(&label);

    let button_width = 120;
    let button_height = 30;
    button_sleep.set_size_request(button_width, button_height);
    button_hibernate.set_size_request(button_width, button_height);
    button_shutdown.set_size_request(button_width, button_height);
    button_ignore.set_size_request(button_width, button_height);

    let gtk_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_bottom(10)
        .margin_top(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    gtk_box.set_size_request(500, 100);

    gtk_box.append(&button_sleep);
    gtk_box.append(&button_hibernate);
    gtk_box.append(&button_shutdown);
    gtk_box.append(&button_ignore);

    let center_container = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    center_container.append(&gtk_box_parent);
    center_container.append(&gtk_box);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Power is out")
        .default_width(600)
        .default_height(200)
        .child(&center_container)
        .build();

    button_ignore.connect_clicked(clone!(
        #[strong]
        window,
        move |_| {
            close_app(&window, "ignore");
        }
    ));

    button_sleep.connect_clicked(clone!(
        #[strong]
        window,
        move |_| {
            close_app(&window, "systemctl suspend");
        }
    ));

    button_hibernate.connect_clicked(clone!(
        #[strong]
        window,
        move |_| {
            close_app(&window, "systemctl hibernate");
        }
    ));

    button_shutdown.connect_clicked(clone!(
        #[strong]
        window,
        move |_| {
            close_app(&window, "systemctl poweroff");
        }
    ));

    if env::var("WINDOW_MOD").as_deref() == Ok("fullscreen") {
        window.fullscreen();
    }

    window.present()
}

fn default() {
    let default = env::var("DEFAULT").unwrap_or_else(|_| "shutdown".to_string());
    let command = format!("systemctl {}", core::get_default(&default));

    match core::run_command(&command) {
        Ok(result) => println!("{}", result),
        Err(err) => {
            eprintln!("Error executing command: {}", err)
        }
    }
}

fn close_app(app: &ApplicationWindow, action: &str) {
    match core::run_command(action) {
        Ok(result) => println!("{}", result),
        Err(err) => {
            eprintln!("Error executing command: {}", err)
        }
    }
    app.close();
}
