use gtk::prelude::*;
use gtk::{self, glib, Application, ApplicationWindow, Button, Label, Orientation};
use std::rc::Rc;
use upsync::core;

const APP_ID: &str = "com.dhanu.upsync";

pub fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(popup);

    app.run()
}

fn popup(app: &Application) {
    let label = Label::builder().label(&get_default()).build();

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

    let app = Rc::new(app.clone());
    button_ignore.connect_clicked({
        let app = Rc::clone(&app);
        move |_| close_app(&app, "ignore")
    });

    button_hibernate.connect_clicked({
        let app = Rc::clone(&app);
        move |_| close_app(&app, "hibernate")
    });

    button_sleep.connect_clicked({
        let app = Rc::clone(&app);
        move |_| close_app(&app, "suspend")
    });

    button_shutdown.connect_clicked({
        let app = Rc::clone(&app);
        move |_| close_app(&app, "poweroff")
    });

    window.present()
}

fn get_default() -> String {
    format!(
        "System will {} in {} seconds. Click 'Ignore' to cancel.",
        upsync::core::get_env("DEFAULT_BEHAVIOUR"),
        upsync::core::get_env("SEC")
    )
}

fn close_app(app: &Rc<gtk::Application>, action: &str) {
    println!("{action}");
    let action = format!("systemctl {}", action);
    let output = core::run_command(&action);
    match output {
        // need to implement proper error handling
        Ok(result) => println!("{}", result),
        Err(err) => {
            println!("{}", err)
        }
    }
    app.quit();
}
