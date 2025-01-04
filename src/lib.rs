pub mod state {
    use battery;
    use std::{env, fs, io, process, thread, time};

    pub struct ClientAction {
        pub command: String,
        pub action: bool,
    }

    pub struct Ssh {
        pub user_name: String,
        pub ip: String,
        pub key: String,
        pub command: String,
    }

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

    pub fn clinet_communication(clientaction: &ClientAction) -> Result<bool, io::Error> {
        let output = match clientaction.action {
            true => process::Command::new("sh")
                .arg("-c")
                .arg(&clientaction.command)
                .output()?,
            false => process::Command::new("sh")
                .arg("-c")
                .arg(&clientaction.command)
                .output()?,
        };

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

    pub fn get_config() -> (String, ClientAction) {
        let option = env::args()
            .skip(1)
            .next()
            .unwrap_or_else(|| "default".to_string());

        let command = ClientAction {
            command: "ping -c 1 -W 1 127.0.0.1".to_string(),
            action: true,
        };

        (option, command)
    }

    pub fn run(inputs: (String, ClientAction)) {
        let (input, command) = inputs;
        match input.as_str() {
            "setup" => unimplemented!("setup"),
            "server" => run_server(command),
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

    pub fn run_server(command: ClientAction) {
        loop {
            match battery_present() {
                Ok(state) => println!("Battery state: {}", state),
                Err(e) => eprintln!("Error: {}", e),
            }

            println!("{}", clinet_communication(&command).unwrap());

            if command.action {
                let _ = exigute_ssh(read_file_for_testing());
            }

            thread::sleep(time::Duration::from_secs(1));
        }
    }
}
