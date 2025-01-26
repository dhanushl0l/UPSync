use glib::timeout_add_seconds;
use gtk::prelude::*;
use gtk::{self, glib, Application, ApplicationWindow, Button, Label, Orientation};
use std::error::Error;
use tokio::{io::AsyncReadExt, net::TcpListener};
use upsync::core;

const APP_ID: &str = "com.dhanu.upsync";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //alpha code
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];

            match socket.read(&mut buffer).await {
                Ok(n) => {
                    let received = String::from_utf8_lossy(&buffer[..n]);
                    run_gui(received.into_owned());
                }
                Err(e) => {
                    eprintln!("Error reading from socket: {}", e);
                }
            }
        });
    }
}

fn run_gui(defaults: String) -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        popup(app, defaults.clone());

        let sec: u32 = core::get_env("SEC").parse().unwrap_or(30);
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

    let window_ref = window.clone();
    button_ignore.connect_clicked({
        let window = window_ref.clone();
        move |_| close_app(&window, "ignore")
    });

    let window_ref = window.clone();
    button_sleep.connect_clicked({
        let window = window_ref.clone();
        move |_| close_app(&window, "suspend")
    });

    let window_ref = window.clone();
    button_hibernate.connect_clicked({
        let window = window_ref.clone();
        move |_| close_app(&window, "hibernate")
    });

    let window_ref = window.clone();
    button_shutdown.connect_clicked({
        let window = window_ref.clone();
        move |_| close_app(&window, "poweroff")
    });
    window.present()
}

fn default() {
    let action = format!("systemctl {}", upsync::core::get_env("DEFAULT_BEHAVIOUR"));
    let output = core::run_command(&action);

    match output {
        // need to implement proper error handling
        Ok(result) => println!("{}", result),
        Err(err) => {
            println!("{}", err)
        }
    }
}

fn close_app(app: &ApplicationWindow, action: &str) {
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
    app.close();
}
