use crate::core;
use log::{debug, error, info, trace, warn};
use std::{thread, time};

pub fn run_server() {
    if let false = status(core::clinet_state()) {
        offline();
    }
    loop {
        trace!("Main loop!");
        thread::sleep(time::Duration::from_secs(1));
        let battery = match core::battery_present() {
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
    if status(core::clinet_state()) {
        info!("power to back on...");
        std::thread::sleep(std::time::Duration::from_secs(5));
        let battery = match core::battery_present() {
            Ok(state) => state,
            Err(err) => {
                error!("Error: {}", err);
                return;
            }
        };
        match battery {
            battery::State::Discharging => {
                info!("sending command to clinet");
                let ssh_state = core::exigute_ssh(core::read_file_for_testing());
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
        trace!("Ofline state loop");
        std::thread::sleep(std::time::Duration::from_secs(5));
        match core::clinet_state() {
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
