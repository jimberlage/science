#[macro_use]
extern crate clap;
extern crate libc;
extern crate rusqlite;

mod commands;
mod datapoints;
mod migrations;
mod sessions;
mod util;

use clap::App;
use std::fmt::Display;
use std::process;
use util::Error;

fn exit<T>(msg: T, code: i32) where T: Display {
    println!("{}", msg);
    process::exit(code);
}

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => match commands::init() {
            Ok(()) => exit("Initialized repo in .science", 0),
            Err(Error(msg)) => exit(msg, 1),
        },
        ("start", Some(sub_matches)) => {
            let description = sub_matches.value_of("description").unwrap();
            let status = sub_matches.value_of("status").unwrap();

            match commands::start(description, status) {
                Ok((_, _)) => exit("Started experiment.", 0),
                Err(Error(msg)) => exit(msg, 1),
            };
        },
        ("record", Some(sub_matches)) => {
            let description = sub_matches.value_of("description").unwrap();
            let status = sub_matches.value_of("status").unwrap();

            match commands::record(description, status) {
                Ok(_) => exit("Recorded datapoint.", 0),
                Err(Error(msg)) => exit(msg, 1),
            };
        },
        _ => exit("Invalid command.", 1),
    };
}