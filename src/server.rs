use crate::state;
use log::{debug, error, info, trace, warn};
use std::{thread, time};

pub fn run_server() {
    if let false = status(state::clinet_state()) {
        offline();
    }
    loop {
        thread::sleep(time::Duration::from_secs(1));
        let battery = match state::battery_present() {
            Ok(state) => state,
            Err(err) => {
                error!("Error: {}", err);
                return;
            }
        };

        match battery {
            battery::State::Discharging => {
                warn!("device is discharging.");
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

pub fn state_discharging() {
    if status(state::clinet_state()) {
        info!("power to back on...");
        std::thread::sleep(std::time::Duration::from_secs(5));
        let battery = match state::battery_present() {
            Ok(state) => state,
            Err(err) => {
                error!("Error: {}", err);
                return;
            }
        };
        match battery {
            battery::State::Discharging => {
                info!("sending command to clinet");
                let ssh_state = state::exigute_ssh(state::read_file_for_testing());
                ssh_state.unwrap_or_else(|e| {
                    error!("Error during SSH execution: {}", e);
                })
            }
            _ => {
                info!("power is back.");
            }
        }
    } else {
        offline();
    }
}

pub fn offline() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        match state::clinet_state() {
            Ok(true) => {
                info!("client is plugged");
                break;
            }
            Ok(false) => {
                debug!("client is disconected")
            }
            Err(e) => {
                // implement error handling
                warn!("Unable to read the clinet state: {}", e);
            }
        }
    }
}
