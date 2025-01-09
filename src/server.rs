use crate::core;
use log::{debug, error, info, trace, warn};
use std::{process, thread, time};

const APPNAME: &str = "smart-ups";

pub fn run_server() {
    // read data from json
    let config: core::ClientConfig = match core::read_json() {
        Ok(data) => data,
        Err(err) => {
            eprintln!(
                "{} \nPlease run the setup command: `{} setup`.",
                err, APPNAME
            );
            process::exit(1);
        }
    };

    if let false = status(core::client_state(&config)) {
        offline(&config);
    }

    loop {
        trace!("Main loop!");
        thread::sleep(time::Duration::from_secs(1));
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
                state_discharging(&config);
                continue;
            }
            _ => {
                info!("device is charging");
                continue;
            }
        }
    }
}

fn status<E>(result: Result<bool, E>) -> bool
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(e) => {
            error!("Error: {}", e);
            false // If the function fails, it returns false.
        }
    }
}

fn state_discharging(config: &core::ClientConfig) {
    if status(core::client_state(config)) {
        info!("client is online");

        std::thread::sleep(std::time::Duration::from_secs(config.sec));

        let battery = match core::battery_present() {
            Ok(state) => state,
            Err(err) => {
                error!("Unable to read battery status: {}", err);
                return;
            }
        };

        match battery {
            battery::State::Discharging => {
                info!("sending command to client");
                let ssh_state = core::exigute_ssh(config);
                let output = ssh_state.unwrap_or_else(|e| {
                    error!("Error during SSH execution: {}", e);
                    e.to_string()
                });
                info!("{}", output);
            }
            _ => {
                info!("power is back.");
            }
        }
    } else {
        offline(config);
    }
}

fn offline(config: &core::ClientConfig) {
    loop {
        trace!("Ofline state loop");
        std::thread::sleep(std::time::Duration::from_secs(5));
        match core::client_state(config) {
            Ok(true) => {
                info!("client is online");
                break;
            }
            Ok(false) => {
                debug!("client is offline")
            }
            Err(e) => {
                // implement error handling
                warn!("Unable to read the client state: {}", e);
            }
        }
    }
}
