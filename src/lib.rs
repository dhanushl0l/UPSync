mod server;
pub mod state {
    use crate::server;
    use battery;
    use std::{env, fs, io, process};

    pub struct Ssh {
        pub user_name: String,
        pub ip: String,
        pub key: String,
        pub command: String,
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
            .arg("ping -c 1 -W 1 192.168.1.106")
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

    pub fn run(inputs: String) {
        match inputs.as_str() {
            "setup" => unimplemented!("setup"),
            "server" => server::run_server(),
            "client" => unimplemented!("gui app"),
            _ => println!(
                r#"Smart-UPS: Convert a non-smart UPS into a smart UPS using laptop power state.

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
