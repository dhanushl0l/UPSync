use env_logger::{Builder, Env};
use upsync::core;

fn main() {
    let env = Env::default().filter_or("LOG", "info");
    Builder::from_env(env).init();

    // Everything in here is temporary for debugging and testing.
    core::run(core::get_args());
}
