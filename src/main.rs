use env_logger;
use smart_ups::core;
fn main() {
    env_logger::init();
    // Everything in here is temporary for debugging and testing.
    core::run(core::get_args());
}
