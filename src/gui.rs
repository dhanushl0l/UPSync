use gtk::prelude::*;
use gtk::{self, glib, Application, ApplicationWindow, Button, Orientation};
use std::io;
use std::process::{Command, Output};
use std::rc::Rc;

const APP_ID: &str = "com.dhanu.upsync";

pub fn client_ui() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(popup);

    app.run()
}

fn popup(app: &Application) {
    let button_sleep = Button::builder().label("Sleep").build();
    let button_hibernate = Button::builder().label("Hibernate").build();
    let button_shutdown = Button::builder().label("Shutdown").build();
    let button_exit = Button::builder().label("Exit").focus_on_click(true).build();

    let button_width = 120;
    let button_height = 30;
    button_sleep.set_size_request(button_width, button_height);
    button_hibernate.set_size_request(button_width, button_height);
    button_shutdown.set_size_request(button_width, button_height);
    button_exit.set_size_request(button_width, button_height);

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
    gtk_box.append(&button_exit);

    let center_container = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    center_container.append(&gtk_box);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Power is out")
        .default_width(600)
        .default_height(200)
        .child(&center_container)
        .build();

    let app = Rc::new(app.clone());
    button_exit.connect_clicked({
        let app = Rc::clone(&app);
        move |_| close_app(&app, "exit")
    });

    button_hibernate.connect_clicked({
        let app = Rc::clone(&app);
        move |_| close_app(&app, "hibernate")
    });

    button_sleep.connect_clicked({
        let app = Rc::clone(&app);
        move |_| close_app(&app, "sleep")
    });

    button_shutdown.connect_clicked({
        let app = Rc::clone(&app);
        move |_| close_app(&app, "shutdown")
    });

    window.present()
}

pub fn close_app(app: &Rc<gtk::Application>, action: &str) {
    println!("{action}");
    let a = match action {
        "exit" => put_to(action),
        "hibernate" => put_to(action),
        "sleep" => put_to(action),
        "shutdown" => put_to(action),
        _ => put_to(action),
    };

    match a {
        Ok(result) => println!("{:?}", result),
        Err(err) => println!("{}", err),
    }

    app.quit();
}

fn put_to(action: &str) -> Result<Output, io::Error> {
    let output = Command::new("sh").arg("-c").arg(action).output()?;
    Ok(output)
}
