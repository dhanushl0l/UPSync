use crate::core;
use log::{debug, error, info, trace, warn};
use std::error::Error;
use std::sync::OnceLock;
use std::{process, thread, time};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const APPNAME: &str = "upsync";
static CONFIG: OnceLock<core::ClientConfig> = OnceLock::new();

fn get_config() -> &'static core::ClientConfig {
    // read data from json once to avoide any unxpected errors,
    CONFIG.get_or_init(|| match core::read_json("config.json") {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{} \nPlease run the setup command: {} setup.", err, APPNAME);
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
        thread::sleep(time::Duration::from_secs(5));
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
                // Wait for 30 seconds before sending the popup to the client
                thread::sleep(time::Duration::from_secs(5));
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

        // std::thread::sleep(std::time::Duration::from_secs(get_config().sec));

        let battery = match core::battery_present() {
            Ok(state) => state,
            Err(err) => {
                error!("Unable to read battery status: {}", err);
                debug!("Returning charging as default");
                battery::State::Charging
            }
        };

        match battery {
            battery::State::Discharging => {
                info!("sending command to client");
                wait_for_power();
            }
            _ => {
                info!("power is back.");
            }
        }
    } else {
        offline();
    }
}

#[tokio::main]
async fn send_device_to() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(&get_config().ip).await?;

    let default = format!(
        "The device will automatically {:?} in {:?} seconds. Click 'Ignore' to cancel.",
        get_config().default_behaviour,
        get_config().sec
    );
    // Write some data.
    stream.write_all(default.as_bytes()).await?;
    Ok(())
}

fn wait_for_power() {
    match send_device_to() {
        Ok(()) => info!("popup open surcess"),
        Err(err) => error!("popup open error: {}", err),
    }
    loop {
        trace!("wait_for_power loop");
        std::thread::sleep(std::time::Duration::from_secs(5));

        let battery = match core::battery_present() {
            Ok(state) => state,
            Err(err) => {
                error!("Unable to read battery status: {}", err);
                debug!("Returning charging as default");
                continue;
            }
        };

        match battery {
            battery::State::Discharging => {
                warn!("device is discharging.");
                continue;
            }
            _ => {
                info!("device is charging and power is back");
                wake_the_pc();
                break;
            }
        }
    }
}

fn offline() {
    loop {
        trace!("Ofline state loop");
        std::thread::sleep(std::time::Duration::from_secs(5));
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

    let wol = core::run_command(&command);

    match wol {
        Ok(result) => {
            if result {
                info!("WOL command succeeded!");
            } else {
                error!("WOL command failed!");
            }
        }
        Err(err) => {
            error!("error sending wol {}", err);
            warn!(
                "Verify the ip and mac address of the client and run '{} setup'",
                APPNAME
            );
        }
    }
}
