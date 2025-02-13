use crate::core;
use serde_json::to_writer;
use std::env;
use std::path::PathBuf;
use std::{fs::create_dir_all, fs::File, process};

pub fn server_setup() {
    println!("Are you sure you want to delete the existing config and start creating a new config? (y/n)");
    let input = core::user_input().unwrap().to_lowercase();
    match input.as_str() {
        "y" => gen_json(),
        "n" => process::exit(0),
        _ => {
            println!("Please enter a valid option.");
            process::exit(1);
        }
    }
}

fn gen_json() {
    use core::ClientConfig;
    let config = ClientConfig::new();
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let config_path: PathBuf = [home.as_str(), ".local/share/upsync/config.json"]
        .iter()
        .collect();

    create_dir_all(config_path.parent().unwrap()).unwrap();

    println!("File created successfully!");
    let file = File::create(config_path).unwrap();
    to_writer(file, &config).unwrap();
}
