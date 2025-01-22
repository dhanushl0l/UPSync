mod server;
mod setup;
pub mod core {
    use crate::{server, setup};
    use battery;
    use std::{env, error::Error, fs, io, process};
    use tokio::net::TcpStream;

    const APPNAME: &str = "upsync";

    // This enum represents the JSON structure
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ClientConfig {
        pub username: String,
        pub key: String,
        pub ip: String,
        pub mac_address: String,
        pub sec: u64,
        pub popup: bool,
        pub default_behaviour: u8,
    }

    impl ClientConfig {
        pub fn new() -> ClientConfig {
            ClientConfig {
                username: ClientConfig::parse_input_string("Enter the username of your local device: "),
                key: ClientConfig::parse_input_string("Enter the ssh key: "),
                ip: ClientConfig::parse_input_string("Enter the IP address of your device (e.g., 192.168.66.99:22 or [::1]:22)"),
                mac_address: ClientConfig::parse_input_string("Enter the MacAddress of your device: "),
                sec: ClientConfig::parse_input_u64("Enter the time (in seconds) after power loss to put the device to sleep: \nDefault: 30"),
                popup: ClientConfig::get_yes_no_input("Do you want to see the popup when power is out? (y/n): \nDefault: y"),
                default_behaviour: ClientConfig::parse_input_u8("Default behaviour when power is out.\n1 = Sleep\n2 = Shutdown\n3 = Hybernate\n4 = Do nothing \nDefault: 1"),
            }
        }

        fn get_input(prompt: &str) -> String {
            println!("{}", prompt);
            self::user_input().expect("Failed to read input")
        }

        fn parse_input_string(prompt: &str) -> String {
            let mut attempts = 0;

            while attempts < 3 {
                let input = ClientConfig::get_input(prompt);

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
                let input = ClientConfig::get_input(prompt);

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
                let input = self::user_input().unwrap_or_default().trim().to_lowercase();
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

        fn parse_input_u8(prompt: &str) -> u8 {
            let mut attempts = 0;

            while attempts < 3 {
                let input = ClientConfig::get_input(prompt);

                if input.is_empty() {
                    return 1;
                }

                match input.parse::<u8>() {
                    Ok(x) if x >= 1 && x <= 4 => return x,
                    _ => {
                        eprintln!("Invalid input. Please enter a choice between 1 and 4.");
                        attempts += 1;
                    }
                }
            }
            println!("Exceeded maximum attempts.");
            std::process::exit(1);
        }
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

    // This function is not yet optimized to perform as expected.
    use ssh2::Session;
    use std::io::prelude::*;

    pub fn exigute_ssh(data: &ClientConfig) -> Result<String, Box<dyn std::error::Error>> {
        let ip = format!("{}:22", data.ip);
        let tcp = std::net::TcpStream::connect(&ip)?;
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        sess.userauth_password(&data.username, &data.key)?;
        assert!(sess.authenticated());

        let mut channel = sess.channel_session()?;

        let command = format!(
            // Need to implement automatic session and display identifier.
            "export WAYLAND_DISPLAY=wayland-1 && DEFAULT_BEHAVIOUR={} && SEC={} &&upsync-gui",
            data.default_behaviour, data.sec
        );
        channel.exec(&command)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        let _ = channel.wait_close()?;
        println!("{} : {}", channel.exit_status()?, s);

        Ok(s)
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

    pub fn read_json() -> Result<ClientConfig, Box<dyn std::error::Error>> {
        let path = "config.json";

        let data = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let json: ClientConfig =
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
