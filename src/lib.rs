pub mod state {
    use battery;
    use std::{io, process::Command};

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

    pub fn client_state(command: String) -> Result<bool, io::Error> {
        let output = Command::new("sh").arg("-c").arg(command).output()?;

        Ok(output.status.success())
    }
}
