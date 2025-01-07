use crate::core;
use serde_json::to_writer;
use std::{fs::File, process};

pub fn server_setup() {
    println!("Are you sure you want to delete the existing config and start creating a new config? (y/n)");
    let input = core::user_input().unwrap().to_lowercase();
    match input.as_str() {
        "y" => continue_setup(),
        "n" => process::exit(0),
        _ => {
            println!("Please enter a valid option.");
            process::exit(1);
        }
    }
}

fn continue_setup() {
    // TODO: Implement proper error handling here
    gen_json();
}

fn gen_json() {
    use core::ClientConfig;
    let a = ClientConfig::new();

    let file = File::create("config.json").unwrap();
    to_writer(file, &a).unwrap();
}
