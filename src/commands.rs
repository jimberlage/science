use migrations;
use models::{Datapoint, Experiment};
use std::fmt::Display;
use std::process;
use util::{git_commit, lookup_git_sha, mkdir, new_conn, specific_error, Error, PROJECT_DIR, Result};

pub type CommandResult = Result<String>;

fn exit<T>(msg: T, code: i32) where T: Display {
    println!("{}", msg);
    process::exit(code);
}

fn run_init() -> Result<()> {
    try_generic!(mkdir(PROJECT_DIR));

    let conn = try_generic!(new_conn());

    migrations::run(&conn)
}

pub fn init() {
    match run_init() {
        Ok(()) => exit("Initialized science project in .science directory.", 0),
        Err(err) => exit(err, 1),
    }
}

pub fn start(description: &str, status: &str) -> CommandResult {
    let owned_description = String::from(description);
    let owned_status = String::from(status);
    let mut conn = try_and_log_generic!(new_conn());
    let opt_experiment = try_and_log_generic!(Experiment::current(&conn));

    match opt_experiment {
        Some(_) => {
            let err: Option<String> = None;
            let msg = String::from("A science experiment is already in progress.  To record a new datapoint, run `science record`.");
            Err(specific_error(err, msg))
        },
        None => {
            let tx = try_and_log_generic!(conn.transaction());
            let session = try_and_log_generic!(Experiment::create(&tx));

            try!(session.make_current(&tx));

            let sha = try_and_log_generic!(lookup_git_sha());
            let _ = try_and_log_generic!(Datapoint::create(&tx, &owned_description, session.id, &sha, &owned_status));

            try_and_log_generic!(tx.commit());

            Ok(String::from("Started experiment."))
        },
    }
}

pub fn record(description: &str, status: &str) -> CommandResult {
    let owned_description = String::from(description);
    let owned_status = String::from(status);
    let conn = try_and_log_generic!(new_conn());
    let opt_session = try_and_log_generic!(Experiment::current(&conn));

    match opt_session {
        Some(session) => {
            try_and_log_generic!(git_commit(description, status));

            // Once we've successfully committed, rollback is tricky.  We don't attempt to persist
            // the datapoint again, since having a DB error means that there is an increased
            // likelihood of another DB error when retrying.  Instead, users can rely on the log in
            // .science/client.log, and revert the commit if they choose.
            let sha = try_and_log_generic!(lookup_git_sha());

            let _ = Datapoint::create(&conn, &owned_description, session.id, &sha, &owned_status);

            Ok(String::from("Recorded datapoint."))
        },
        None => {
            let err: Option<String> = None;
            let msg = String::from("You need to start a science experiment first.  Run `science start`.");
            Err(Error::Specific(err, msg))
        },
    }
}

pub fn stop() -> CommandResult {
    let mut conn = try_and_log_generic!(new_conn());
    let opt_session = try_and_log_generic!(Experiment::current(&conn));

    match opt_session {
        Some(session) => {
            // We create a transaction here, as there's a chance that the session will need to be
            // deleted from two places, the sessions table and the current_session table.
            let tx = try_and_log_generic!(conn.transaction());

            try_and_log_generic!(session.delete(&tx));

            try_and_log_generic!(tx.commit());

            Ok(String::from("Stopped experiment."))
        },
        None => {
            let err: Option<String> = None;
            let msg = String::from("There is no ongoing science experiment to stop.");
            Err(specific_error(err, msg))
        },
    }
}

pub fn analyze() {
}
