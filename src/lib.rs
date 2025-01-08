mod server;
mod setup;
pub mod core {
    use crate::{server, setup};
    use battery;
    use std::{env, fs, io, process};

    pub struct Ssh {
        pub user_name: String,
        pub ip: String,
        pub key: String,
        pub command: String,
    }

    // This enum represents the JSON structure
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ClientConfig {
        username: String,
        key: String,
        ip: String,
        mac_address: String,
        sec: i32,
        popup: bool,
        default: u8,
    }

    impl ClientConfig {
        pub fn new() -> ClientConfig {
            ClientConfig {
                username: ClientConfig::parse_input_string("Enter the username of your local device: "),
                key: ClientConfig::parse_input_string("Enter the key: "),
                ip: ClientConfig::parse_input_string("Enter the ip of your device: "),
                mac_address: ClientConfig::parse_input_string("Enter the MacAddress of your device: "),
                sec: ClientConfig::parse_input_i32("Enter the time (in seconds) after power loss to put the device to sleep: \nDefault: 30"),
                popup: ClientConfig::get_yes_no_input("Do you want to see the popup when power is out? (y/n): \nDefault: y"),
                default: ClientConfig::parse_input_u8("Default behaviour when power is out.\n1 = Sleep\n2 = Shutdown\n3 = Hybernate\n4 = Do nothing \nDefault: 1"),
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

        fn parse_input_i32(prompt: &str) -> i32 {
            let mut attempts = 0;

            while attempts < 3 {
                let input = ClientConfig::get_input(prompt);

                if input.is_empty() {
                    return 30;
                }

                match input.parse::<i32>() {
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

    pub fn clinet_state() -> Result<bool, io::Error> {
        let output = process::Command::new("sh")
            .arg("-c")
            .arg("ping -c 1 -W 1 192.168.1.69")
            .output()?;

        Ok(output.status.success())
    }

    pub fn read_file_for_testing() -> Ssh {
        let ssh = fs::read_to_string("ssh.txt").expect("unable to read");
        let mut ssh_vec: Vec<&str> = Vec::new();
        for data in ssh.lines() {
            ssh_vec.push(data);
        }

        Ssh {
            user_name: ssh_vec[0].to_string(),
            ip: ssh_vec[1].to_string(),
            key: ssh_vec[2].to_string(),
            command: ssh_vec[3].to_string(),
        }
    }

    // This function is not yet optimized to perform as expected.
    use ssh2::Session;
    use std::io::prelude::*;
    use std::net::TcpStream;

    pub fn exigute_ssh(ssh: Ssh) -> Result<(), Box<dyn std::error::Error>> {
        let tcp = TcpStream::connect(&ssh.ip)?;
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        sess.userauth_password(&ssh.user_name, &ssh.key)?;
        assert!(sess.authenticated());

        let mut channel = sess.channel_session()?;
        channel.exec(&ssh.command)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        println!("{}", s);
        let _ = channel.wait_close();
        println!("{}", channel.exit_status()?);

        Ok(())
    }

    pub fn get_args() -> String {
        env::args()
            .skip(1)
            .next()
            .unwrap_or_else(|| "default".to_string())
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
            "client" => unimplemented!("gui app"),
            _ => println!(
                r#"Smart-UPS: Convert a non-smart UPS into a smart UPS using laptop power states.

Usage: smart-ups <command>

Commands:
    setup      Initialize the application (e.g., on a laptop).
    server     Start the power monitoring server.
    client     Run this on the client to see the demo popup.
"#
            ),
        }
    }
}
