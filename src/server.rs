use crate::core;
use log::{debug, error, info, trace, warn};
use std::sync::OnceLock;
use std::{process, thread, time};

const APPNAME: &str = "smart-ups";
static CONFIG: OnceLock<core::ClientConfig> = OnceLock::new();

fn get_config() -> &'static core::ClientConfig {
    // read data from json once to avoide any unxpected errors,
    CONFIG.get_or_init(|| match core::read_json() {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{} \nPlease run the setup command: {} setup.", err, APPNAME);
            process::exit(1);
        }
    })
}

pub fn run_server() {
    let config = format!("ping -c 1 -W 1 {}", get_config().ip);
    if let false = status(core::state(&config)) {
        offline();
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
                state_discharging();
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

fn state_discharging() {
    let config = format!("ping -c 1 -W 1 {}", get_config().ip);

    if status(core::state(&config)) {
        info!("client is online");

        std::thread::sleep(std::time::Duration::from_secs(get_config().sec));

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
                let ssh_state = core::exigute_ssh(get_config());
                let output = ssh_state.unwrap_or_else(|e| {
                    error!("Error during SSH execution: {}", e);
                    e.to_string()
                });
                info!("{}", output);
                parse_user_input(output);
            }
            _ => {
                info!("power is back.");
            }
        }
    } else {
        offline();
    }
}

fn parse_user_input(output: String) {
    let output = output.split_whitespace().next().unwrap_or("");
    match output {
        "ignore" => {
            info!("user {} power state", output);
            wait_for_power();
        }
        "suspend" => {
            info!("user put the device to {}", output);
            is_pc_off(output);
        }
        "hibernate" => {
            info!("user put the device to {}", output);
            is_pc_off(output);
        }
        "shutdown" => {
            info!("user put the device to {}", output);
            is_pc_off(output);
        }
        _ => error!("uexpected error"),
    }
}

fn is_pc_off(option: &str) {
    let mut times = 0;
    const MAX_RETRIES: usize = 10;
    const SLEEP_DURATION: std::time::Duration = std::time::Duration::from_secs(5);
    let config = format!("ping -c 1 -W 1 {}", get_config().ip);

    while times < MAX_RETRIES {
        times += 1;
        std::thread::sleep(SLEEP_DURATION);
        match core::state(&config) {
            Ok(state) => {
                info!(
                    "Waiting for the device to {}... (attempt {}/{})",
                    option, times, MAX_RETRIES
                );
                if state {
                    continue;
                } else {
                    wait_for_power();
                    return;
                }
            }
            Err(err) => {
                error!(
                    "Unable to read device state: {}. Retrying... (attempt {}/{})",
                    err, times, MAX_RETRIES
                );
            }
        }
    }
    wait_for_power()
}

fn wait_for_power() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));

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
    let config = format!("ping -c 1 -W 1 {}", get_config().ip);

    loop {
        trace!("Ofline state loop");
        std::thread::sleep(std::time::Duration::from_secs(5));
        match core::state(&config) {
            Ok(true) => {
                info!("client is online");
                break;
            }
            Ok(false) => {
                info!("client is offline")
            }
            Err(e) => {
                // implement error handling
                warn!("Unable to read the client state: {}", e);
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

    let wol = core::state(&command);
}
