use smart_ups::state;
fn main() {
    // Everything in here is temporary for debugging and testing.
    let command = state::ClientAction {
        command: "ping -c 1 -W 1 127.0.0.1".to_string(),
        action: true,
    };
    state::run(command);
}
