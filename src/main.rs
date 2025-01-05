use env_logger;
use smart_ups::state;
fn main() {
    env_logger::init();
    // Everything in here is temporary for debugging and testing.
    state::run(state::get_args());
}
