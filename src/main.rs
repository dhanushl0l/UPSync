use smart_ups::state;
fn main() {
    match state::battery_present() {
        Ok(state) => println!("Battery state: {:?}", state),
        Err(e) => eprintln!("Error: {}", e),
    }
}
