use env_logger::{Builder, Env};
use std::env;
use upsync::core;
mod gui;

fn main() {
    if let Ok(value) = env::var("CLIENT") {
        if value == "yes" {
            gui::client_ui();
            return;
        }
    }

    let env = Env::default().filter_or("LOG", "info");
    Builder::from_env(env).init();

    // Everything in here is temporary for debugging and testing.
    core::run(core::get_args());
}
