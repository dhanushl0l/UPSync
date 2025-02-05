use env_logger::{Builder, Env};
use glib::{clone, timeout_add_seconds};
use gtk::prelude::*;
use gtk::{self, glib, Application, ApplicationWindow, Button, Label, Orientation};
use log::{debug, error, info, warn};
use serde_json::to_writer;
use std::{env, error::Error, fs::File, process, sync::OnceLock};
use tokio::{io::AsyncReadExt, net::TcpListener};
use upsync::core;

const APP_ID: &str = "com.dhanu.upsync";

static CONFIG: OnceLock<core::ServerConfig> = OnceLock::new();

fn get_config() -> &'static core::ServerConfig {
    // read data from json once to avoide any unxpected errors,
    CONFIG.get_or_init(|| match core::read_json("Config.json") {
        Ok(data) => data,
        Err(err) => {
            eprintln!(
                "{} \nPlease run the setup command: {} setup.",
                err,
                core::GUI_APPNAME
            );
            process::exit(1);
        }
    })
}

fn main() {
    let env = Env::default().filter_or("LOG", "info");
    Builder::from_env(env).init();

    match env::var("MOD").as_deref() {
        Ok("server") => match run_server() {
            Ok(_) => info!("running server"),
            Err(err) => error!("error opening gui {}", err),
        },
        Ok("gui") => {
            let label = format!("System will shutdown in 30 seconds");
            let exit_code: glib::ExitCode = run_gui(label, 30);
            info!("GUI exited with code: {:?}", exit_code);
        }
        Ok(_) => {
            setup();
        }
        Err(_) => {
            setup();
        }
    }
}

fn setup() {
    println!("Are you sure you want to delete the existing config and start creating a new config? (y/n)");
    let input = core::user_input().unwrap().to_lowercase();
    match input.as_str() {
        "y" => continue_setup(),
        "n" => process::exit(0),
        _ => {
            println!("Please enter a valid option.");
            process::exit(1);
        }
    }
}

fn continue_setup() {
    use core::ServerConfig;
    let config = ServerConfig::new();

    let file = File::create("Config.json").unwrap();
    to_writer(file, &config).unwrap();
}

#[tokio::main]
async fn run_server() -> Result<(), Box<dyn Error>> {
    let ip = format!("0.0.0.0:{}", get_config().ip);
    let listener = TcpListener::bind(&ip).await?;
    info!("Server started on {}", ip);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        debug!("New connection from {}", addr);

        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];

            match socket.read(&mut buffer).await {
                Ok(n) => {
                    let received = String::from_utf8_lossy(&buffer[..n]).to_string();

                    let message: String = format!(
                        "The device will automatically {:?} in {} seconds. Click 'Ignore' to cancel.",
                        get_config().default_behaviour,
                        get_config().default_delay,
                    );
                    if received == get_config().key {
                        info!("Valid key received from {}", addr);
                        run_gui(message, get_config().default_delay);
                    } else if !received.is_empty() {
                        warn!("Invalid key received from {}", addr);
                    }
                }
                Err(e) => {
                    error!("Error reading from socket: {}", e);
                }
            }
        });
    }
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

    window.present()
}

fn default() {
    let output = if env::var("MOD").as_deref() == Ok("server") {
        core::run_command(&core::get_default(&get_config().default_behaviour))
    } else {
        core::run_command("systemctl poweroff")
    };

    match output {
        Ok(result) => info!("{}", result),
        Err(err) => {
            error!("Error executing command: {}", err)
        }
    }
}

fn close_app(app: &ApplicationWindow, action: &str) {
    let output = core::run_command(action);
    match output {
        Ok(result) => debug!("{}", result),
        Err(err) => {
            error!("Error executing command: {}", err)
        }
    }
    app.close();
}
