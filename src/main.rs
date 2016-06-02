#[macro_use]
extern crate clap;
extern crate libc;
extern crate rusqlite;

#[macro_use]
mod util;
mod commands;
mod migrations;
mod models;

use clap::App;
use std::fmt::Display;
use std::process::exit;

fn exit_with_error<T>(msg: T) where T: Display {
    println!("{}", msg);
    exit(1);
}

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => commands::init(),
        ("start", Some(sub_matches)) => {
            let description = sub_matches.value_of("description").unwrap();
            let status = sub_matches.value_of("status").unwrap();

            commands::start(description, status);
        },
        ("record", Some(sub_matches)) => {
            let description = sub_matches.value_of("description").unwrap();
            let status = sub_matches.value_of("status").unwrap();

            commands::record(description, status);
        },
        ("stop", Some(_)) => commands::stop(),
        ("analyze", Some(_)) => commands::analyze(),
        _ => exit_with_error("Invalid command."),
    };
}
