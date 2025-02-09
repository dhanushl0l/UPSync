mod server;
mod setup;

pub mod core {
    use battery;
    use std::{env, error::Error, fs, io, process};
    use tokio::net::TcpStream;
    // This enum represents the JSON structure
    use serde::{Deserialize, Serialize};

    use crate::{server, setup};

    pub const APPNAME: &str = "upsync";
    pub const GUI_APPNAME: &str = "upsync-gui";

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ClientConfig {
        pub user: String,
        pub key: String,
        pub ip: String,
        pub wake: bool,
        pub mac_address: String,
        default_behaviour: Behaviour,
        default_delay: u32,
        pub default_action_delay: u64,
        pub popup: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Behaviour {
        Sleep,
        Hybernate,
        Shutdown,
        Ignore,
    }

    impl ClientConfig {
        pub fn new() -> ClientConfig {
            ClientConfig {
                user: parse_input_string("Enter the client username: ",false),
                key: parse_input_string("Enter the user password: ",false),
                ip: parse_input_string("Enter the IP address of your device with the port (e.g., 192.168.66.99:9898): )", false),
                wake: get_yes_no_input("Do you want to wake the PC automatically when the power is restored using WOL (Wake-on-LAN)? (y/n) [Default: n]: ",false),
                mac_address: parse_input_string("Enter the MAC address of your device (Leave blank if you did not choose to enable Wake-on-LAN): ",true),   
                default_delay:parse_input_u32("Enter the time (in seconds) after power loss to put the device to default behaviour: \nDefault: 30"),    
                default_behaviour: parse_input_behaviour("Default behaviour when power is out: \n1 = Sleep\n2 = Hybernate\n3 = Shutdown\n4 = Do nothing \nDefault: 1 "),         
                default_action_delay: 5,
                popup: get_yes_no_input("Do you want to see the popup when power is out? (y/n): \nDefault: y", true),
            }
        }
    }

    fn get_input(prompt: &str) -> String {
        println!("{}", prompt);
        user_input().expect("Failed to read input")
    }

    fn parse_input_string(prompt: &str, act: bool) -> String {
        let mut attempts = 0;

        while attempts < 3 {
            let input = get_input(prompt);

            if input.is_empty() && !act {
                eprintln!("Invalid input.");
                attempts += 1;
                continue;
            }
            return input;
        }

        println!("Exceeded maximum attempts.");
        std::process::exit(1);
    }

    fn parse_input_u32(prompt: &str) -> u32 {
        let mut attempts: u64 = 0;

        while attempts < 3 {
            let input = get_input(prompt);

            if input.is_empty() {
                return 30;
            }

            match input.parse::<u32>() {
                Ok(x) => return x,
                Err(_) => {
                    eprintln!(
                        "Invalid input. Please enter a valid number of seconds as an integer."
                    );
                    attempts += 1;
                }
            }
        }

        println!("Exceeded maximum attempts. Exiting or using default value.");
        std::process::exit(1);
    }

    fn get_yes_no_input(prompt: &str, default: bool) -> bool {
        let mut attempts = 0;

        while attempts < 3 {
            println!("{}", prompt);
            let input = user_input().unwrap_or_default().trim().to_lowercase();
            match input.as_str() {
                // Default to "yes" if the user presses Enter
                "" => return default,
                "y" => return true,
                "n" => return false,
                _ => {
                    eprintln!("Invalid input. Please enter 'y' for yes or 'n' for no.");
                    attempts += 1;
                }
            }
        }

        println!("Exceeded maximum attempts.");
        std::process::exit(1);
    }

    fn parse_input_behaviour(prompt: &str) -> Behaviour {
        let mut attempts = 0;

        while attempts < 3 {
            let input = get_input(prompt);

            if input.is_empty() {
                return Behaviour::Sleep;
            }

            match input.parse::<u8>() {
                Ok(1) => return Behaviour::Sleep,
                Ok(2) => return Behaviour::Hybernate,
                Ok(3) => return Behaviour::Shutdown,
                Ok(4) => return Behaviour::Ignore,
                _ => {
                    eprintln!("Invalid input. Please enter a choice between 1 and 4.");
                    attempts += 1;
                }
            }
        }
        println!("Exceeded maximum attempts.");
        std::process::exit(1);
    }

    // On my laptop, if the battery is full, it reports "unknown" instead of "full."
    // As a workaround, run_server() assumes "unknown" means the battery is charging.
    pub fn battery_present() -> Result<battery::State, battery::Error> {
        let manager = battery::Manager::new()?;

        let battery = match manager.batteries()?.next() {
            Some(Ok(battery)) => battery,
            Some(Err(e)) => return Err(e),
            None => {
                return Err(io::Error::from(io::ErrorKind::NotFound).into());
            }
        };
        return Ok(battery.state());
    }

    pub fn run_command(config: &str) -> Result<bool, io::Error> {
        let command = config;
        let output = process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        Ok(output.status.success())
    }

    #[tokio::main]
    pub async fn device_status(ip: &str) -> Result<bool, Box<dyn Error>> {
        match TcpStream::connect(ip).await {
            Ok(_) => Ok(true),
            // Return false on error assuming the client is down.
            Err(_) => Ok(false),
        }
    }

    pub fn connect_to_client() -> Result<String, String> {
        unimplemented!()
    }

    pub fn get_args() -> String {
        env::args()
            .skip(1)
            .next()
            .unwrap_or_else(|| "default".to_string())
    }

    pub fn get_env(env: &str) -> String {
        match env::var(env) {
            Ok(val) => val,
            Err(_) => "error".to_string(),
        }
    }

    pub fn read_json<T: for<'de> Deserialize<'de>>(
        path: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let json: T =
            serde_json::from_str(&data).map_err(|e| format!("Failed to parse JSON: {}", e))?;
        Ok(json)
    }

    pub fn user_input() -> Result<String, io::Error> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    pub fn get_default(action: &str) -> String {
        match action {
            "suspend" => String::from("suspend"),
            "hybernate" => String::from("hibernate"),
            "shutdown" => String::from("poweroff"),
            _ => String::from(""),
        }
    }

    pub fn run(inputs: String) {
        match inputs.as_str() {
            "setup" => setup::server_setup(),
            "server" => server::run_server(),
            _ => {
                println!(
                    r#"{}: Convert a non-smart UPS into a smart UPS using laptop power states.
    
    Usage: {} <command>
    
    Commands:
    setup      Initialize the application (e.g., on a laptop).
    server     Start the power monitoring server.
    client     Run this on the client to see the demo popup.
    "#,
                    APPNAME, APPNAME
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokio() {
        let state = match core::device_status("127.0.0.1:22") {
            Ok(state) => state,
            Err(err) => {
                eprint!("{}", err);
                false
            }
        };
        assert!(state)
    }
}
