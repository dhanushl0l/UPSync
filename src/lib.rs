mod server;
mod setup;

pub mod core {
    use battery;
    use std::{env, error::Error, fs, io, process};
    use tokio::net::TcpStream;
    // This enum represents the JSON structure
    use serde::{Deserialize, Serialize};

    use crate::{server, setup};

    const APPNAME: &str = "upsync";

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ClientConfig {
        pub key: String,
        pub ip: String,
        pub mac_address: String,
        pub sec: u64,
        pub default_behaviour: Behaviour,
        pub platform: Platform,
        pub popup: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Behaviour {
        Sleep,
        Hybernate,
        Shutdown,
        Ignore,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Platform {
        Linux,
        Windows,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ServerConfig {
        pub ip: String,
        pub key: String,
    }

    impl ServerConfig {
        pub fn new() -> ServerConfig {
            ServerConfig {
                key: get_key("Create a secure key (minimum 8 characters)"),
                ip: parse_input_string("Enter the port no the server is running (e.g., 9898)"),
            }
        }
    }

    impl ClientConfig {
        pub fn new() -> ClientConfig {
            ClientConfig {
                key: get_key("Create a secure key (minimum 8 characters)"),
                ip: parse_input_string("Enter the IP address of your device (e.g., 192.168.66.99:22 or [::1]:22)"),
                mac_address: parse_input_string("Enter the MacAddress of your device: "),
                sec: parse_input_u64("Enter the time (in seconds) after power loss to put the device to sleep: \nDefault: 30"),
                default_behaviour: parse_input_behaviour("Default behaviour when power is out.\n1 = Sleep\n2 = Hybernate\n3 = Shutdown\n4 = Do nothing \nDefault: 1"),
                platform: parse_input_platform("What is yout client device platform.\n1 = linux\n2 = Windows"),
                popup: get_yes_no_input("Do you want to see the popup when power is out? (y/n): \nDefault: y"),
            }
        }
    }

    fn get_key(prompt: &str) -> String {
        let mut attempts = 0;

        while attempts < 3 {
            let prompt = parse_input_string(prompt);

            if prompt.len() >= 8 {
                return prompt;
            } else {
                attempts += 1;
                println!("Key must be at least 8 characters long. Try again.");
            }
        }

        panic!("Maximum attempts exceeded. Could not get a valid key.");
    }

    fn get_input(prompt: &str) -> String {
        println!("{}", prompt);
        user_input().expect("Failed to read input")
    }

    fn parse_input_string(prompt: &str) -> String {
        let mut attempts = 0;

        while attempts < 3 {
            let input = get_input(prompt);

            if input.is_empty() {
                eprintln!("Invalid input.");
                attempts += 1;
                continue;
            }
            return input;
        }

        println!("Exceeded maximum attempts.");
        std::process::exit(1);
    }

    fn parse_input_u64(prompt: &str) -> u64 {
        let mut attempts: u64 = 0;

        while attempts < 3 {
            let input = get_input(prompt);

            if input.is_empty() {
                return 30;
            }

            match input.parse::<u64>() {
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

    fn get_yes_no_input(prompt: &str) -> bool {
        let mut attempts = 0;

        while attempts < 3 {
            println!("{}", prompt);
            let input = user_input().unwrap_or_default().trim().to_lowercase();
            match input.as_str() {
                // Default to "yes" if the user presses Enter
                "" => return true,
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

    fn parse_input_platform(prompt: &str) -> Platform {
        let mut attempts = 0;

        while attempts < 3 {
            let input = get_input(prompt);

            match input.parse::<u8>() {
                Ok(1) => return Platform::Linux,
                Ok(2) => return Platform::Windows,
                _ => {
                    eprintln!("Invalid input. Please enter a choice between 1 and 2.");
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
    pub async fn device_status() -> Result<bool, Box<dyn Error>> {
        match TcpStream::connect("80").await {
            Ok(_) => Ok(true),
            // Return false on error assuming the client is down.
            Err(_) => Ok(false),
        }
    }

    pub fn popup_command(data: &ClientConfig) -> String {
        format!(
            "export WAYLAND_DISPLAY=wayland-1 && DEFAULT_BEHAVIOUR={:?} && SEC={} &&upsync-gui",
            data.default_behaviour, data.sec
        )
    }

    pub fn no_popup_command(data: &ClientConfig) -> String {
        format!("systemctl {:?}", data.default_behaviour)
    }

    //
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
        let state = match core::device_status() {
            Ok(state) => state,
            Err(err) => {
                eprint!("{}", err);
                false
            }
        };
        assert!(state)
    }
}
