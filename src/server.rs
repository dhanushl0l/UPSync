use crate::core;
use log::{debug, error, info, trace, warn};
use std::error::Error;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, process, thread, time};

static CONFIG: OnceLock<core::ClientConfig> = OnceLock::new();

fn get_config() -> &'static core::ClientConfig {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let config_path: PathBuf = [home.as_str(), ".local/share/upsync/config.json"]
        .iter()
        .collect();

    // read data from json once to avoide any unxpected errors,
    CONFIG.get_or_init(|| match core::read_json(config_path.as_path()) {
        Ok(data) => data,
        Err(err) => {
            eprintln!(
                "{} \nPlease run the setup command: {} setup.",
                err,
                core::APPNAME
            );
            process::exit(1);
        }
    })
}

pub fn run_server() {
    if let false = status() {
        offline();
    }

    loop {
        trace!("Main loop!");
        thread::sleep(time::Duration::from_secs(get_config().delay_between_tasks));
        let battery = match core::battery_present() {
            Ok(state) => state,
            Err(err) => {
                error!("Error: {}", err);
                continue;
            }
        };

        match battery {
            battery::State::Discharging => {
                warn!("device is discharging.");
                thread::sleep(time::Duration::from_secs(get_config().delay_between_tasks));
                state_discharging();
                continue;
            }
            _ => {
                info!("Device is charging");
                continue;
            }
        }
    }
}

fn status() -> bool {
    match core::device_status(&get_config().ip) {
        Ok(status) => status,
        Err(err) => {
            error!("{}", err);
            info!("Assuming device is offline");
            false
        }
    }
}

fn state_discharging() {
    if status() {
        info!("client is online");

        thread::sleep(time::Duration::from_secs(get_config().delay_between_tasks));

        let battery = match core::battery_present() {
            Ok(state) => state,
            Err(err) => {
                error!("Unable to read battery status: {}", err);
                info!("Returning charging as default");
                battery::State::Charging
            }
        };

        match battery {
            battery::State::Discharging => {
                debug!("Opening popup in client");
                send_device_to();
            }
            _ => {
                info!("power is back.");
            }
        }
    } else {
        offline();
    }
}

fn send_device_to() {
    match get_config().popup {
        true => {
            let command: String = format!(
                "export DISPLAY=:0 && export WAYLAND_DISPLAY=wayland-0 && MOD=gui {}",
                core::GUI_APPNAME
            );
            match run_ssh(command) {
                Ok(()) => info!("popup open surcess"),
                Err(err) => {
                    error!("popup open error: {}", err);
                }
            }
        }

        false => {
            let command = format!(
                "systemctl {}",
                core::get_default_server(&get_config().default_behaviour)
            );
            match run_ssh(command) {
                Ok(()) => info!("popup open surcess"),
                Err(err) => {
                    error!("popup open error: {}", err);
                }
            }
        }
    }

    wait_for_power()
}

use ssh2::Session;
use std::io::Read;
use std::net::TcpStream as TcpStreamSTD;

fn run_ssh(command: String) -> Result<(), Box<dyn Error>> {
    let tcp = TcpStreamSTD::connect("192.168.1.240:22")?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password(&get_config().user, &get_config().key)?;

    let mut channel = sess.channel_session()?;

    channel.exec(&command)?;

    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    parse_result(s);
    channel.wait_close()?;
    Ok(())
}

fn parse_result(s: String) {
    let mut parts = s.split("\n");

    let userchoice = parts.next().unwrap_or("");
    let result = parts.next().unwrap_or("");

    match userchoice {
        "systemctl suspend" => info!("User chose suspend"),
        "systemctl hibernate" => info!("User chose hibernate"),
        "systemctl sleep" => info!("User chose sleep"),
        "ignore" => info!("User chose ignore"),
        _ => error!("Something went wrong"),
    }

    match result {
        "execution surcess" => info!("Command execution surcess"),
        "execution failed" => error!("Command execution failed"),
        _ => error!("Something went wrong"),
    }
}

fn wait_for_power() {
    loop {
        trace!("wait_for_power loop");
        thread::sleep(time::Duration::from_secs(get_config().delay_between_tasks));

        let battery = match core::battery_present() {
            Ok(state) => state,
            Err(err) => {
                error!("Unable to read battery status: {}", err);
                info!("Returning discharging as default");
                battery::State::Discharging
            }
        };

        match battery {
            battery::State::Discharging => {
                info!("Device is discharging. Waiting for power to return.");
                continue;
            }
            _ => {
                info!("Device is charging and power is back");
                if get_config().wake {
                    wake_the_pc();
                }
                break;
            }
        }
    }
}

fn offline() {
    loop {
        trace!("Ofline state loop");
        thread::sleep(time::Duration::from_secs(get_config().delay_between_tasks));
        match status() {
            true => {
                info!("client is online");
                break;
            }
            false => {
                info!("client is offline")
            }
        }
    }
}

fn wake_the_pc() {
    let command = format!(
        "wakeonlan -i {} {}",
        get_config().ip,
        get_config().mac_address
    );

    if !status() {
        info!("Client is ofline");
        thread::sleep(time::Duration::from_secs(get_config().delay_between_tasks));

        let wol = core::run_command(&command);

        match wol {
            Ok(result) => {
                if result {
                    info!("WOL command succeeded!");
                } else {
                    error!("WOL command failed!");
                    info!(
                    "Verify the mac address of the client and run '{} setup' to reconfiger to settings",
                    core::APPNAME
                );
                }
            }
            Err(err) => {
                error!("error sending wol {}", err);
                info!(
                "Verify the mac address of the client and run '{} setup' to reconfiger to settings",
                core::APPNAME
            );
            }
        }
    } else {
        info!("Client is online, skipping WOL.");
    }
}
