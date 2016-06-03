#[macro_use]
extern crate clap;
extern crate rusqlite;

#[macro_use]
mod util;
mod commands;
mod migrations;
mod models;

use clap::App;
use commands::CommandResult;
use std::fmt::Display;
use std::process;

fn exit_with_code<T>(msg: T, code: i32) where T: Display {
    println!("{}", msg);
    process::exit(code);
}

fn exit(cr: CommandResult) {
    match cr {
        Ok(msg) => exit_with_code(msg, 0),
        Err(err) => exit_with_code(err, 1),
    }
}

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => exit(commands::init()),
        ("start", Some(sub_matches)) => {
            let description = sub_matches.value_of("description").unwrap();
            let status = sub_matches.value_of("status").unwrap();

            exit(commands::start(description, status));
        },
        ("record", Some(sub_matches)) => {
            let description = sub_matches.value_of("description").unwrap();
            let status = sub_matches.value_of("status").unwrap();

            exit(commands::record(description, status));
        },
        ("stop", Some(_)) => exit(commands::stop()),
        ("analyze", Some(_)) => exit(commands::analyze()),
        _ => exit_with_code("Invalid command.", 1),
    };
}
