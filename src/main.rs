use smart_ups::state;
fn main() {
    // Everything in here is temporary for debugging and testing.
    match state::battery_present() {
        Ok(state) => println!("Battery state: {}", state),
        Err(e) => eprintln!("Error: {}", e),
    }

    let command = "ping -c 1 -W 1 127.0.0.1".to_string();
    println!("{}", state::client_state(command).unwrap())
}
