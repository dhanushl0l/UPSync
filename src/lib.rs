pub mod state {
    use battery;
    use std::{io, process::Command, thread, time};

    pub struct ClientAction {
        pub command: String,
        pub action: bool,
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
            true => Command::new("sh")
                .arg("-c")
                .arg(&clientaction.command)
                .output()?,
            false => Command::new("sh")
                .arg("-c")
                .arg(&clientaction.command)
                .output()?,
        };

        Ok(output.status.success())
    }

    pub fn run(command: ClientAction) {
        loop {
            thread::sleep(time::Duration::from_secs(1));
            match battery_present() {
                Ok(state) => println!("Battery state: {}", state),
                Err(e) => eprintln!("Error: {}", e),
            }

            println!("{}", clinet_communication(&command).unwrap());
        }
    }
}
