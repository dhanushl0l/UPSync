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
    let a = vec![
        ClientConfig::Username(core::get_input("Enter the username of your local device: ")),
        ClientConfig::Key(core::get_input("Enter the key: ")),
        ClientConfig::Ip(core::get_input("Enter the ip of your device: ")),
        ClientConfig::MacAddress(core::get_input("Enter the MacAddress of your device: ")),
        ClientConfig::Sec(core::parse_input(
            "Enter the time (in seconds) after power loss to put the device to sleep: ")),
        ClientConfig::Popup(core::get_yes_no_input(
            "Do you want to see the popup when power is out? (y/n): ")),
        ClientConfig::Default(core::parse_enum_input(
            "Default behaviour when power is out.\n1 = Sleep\n2 = Shutdown\n3 = Hybernate\n4 = Do nothing: ")),
    ];

    let file = File::create("config.json").unwrap();
    to_writer(file, &a).unwrap();
}
