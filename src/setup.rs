use crate::state;
use serde_json::to_writer;
use std::{fs::File, process};

pub fn server_setup() {
    println!("Are you sure you want to delete the existing config and start creating a new config? (y/n)");
    let input = state::user_input().unwrap().to_lowercase();
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
    gen_json();
}

use state::ClientConfig;
fn gen_json() {
    let a = vec![
        ClientConfig::Username(get_input("Enter the username of your local device: ")),
        ClientConfig::Key(get_input("Enter the key: ")),
        ClientConfig::Ip(get_input("Enter the ip of your device: ")),
        ClientConfig::MacAddress(get_input("Enter the MacAddress of your device: ")),
        ClientConfig::Sec(parse_input(
            "Enter the time (in seconds) after power loss to put the device to sleep: ")),
        ClientConfig::Popup(get_yes_no_input(
            "Do you want to see the popup when power is out? (y/n): ")),
        ClientConfig::Default(parse_enum_input(
            "Default behaviour when power is out.\n1 = Sleep\n2 = Shutdown\n3 = Hybernate\n4 = Do nothing: ")),
    ];

    let file = File::create("config.json").unwrap();
    to_writer(file, &a).unwrap();
}

fn get_input(prompt: &str) -> String {
    println!("{}", prompt);
    state::user_input().unwrap()
}

fn parse_input(prompt: &str) -> i32 {
    let input = get_input(prompt);
    match input.parse::<i32>() {
        Ok(x) => x,
        Err(_) => panic!("Invalid input. Please enter a number."),
    }
}

fn get_yes_no_input(prompt: &str) -> bool {
    println!("{}", prompt);
    match state::user_input().unwrap().to_lowercase().as_str() {
        "y" => true,
        "n" => false,
        _ => {
            println!("Please enter a valid option.");
            process::exit(1);
        }
    }
}

fn parse_enum_input(prompt: &str) -> u8 {
    let input = get_input(prompt);
    match input.parse::<u8>() {
        Ok(x) if x >= 1 && x <= 4 => x,
        _ => {
            panic!("Invalid input. Please enter a choice between 1 and 4.");
        }
    }
}
