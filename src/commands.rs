use migrations;
use models::{Datapoint, Experiment};
use std::fmt::Display;
use std::process;
use util::{git_commit, lookup_git_sha, mkdir, new_conn, Error, PROJECT_DIR_NAME, Result};

fn exit<T>(msg: T, code: i32) where T: Display {
    println!("{}", msg);
    process::exit(code);
}

fn run_init() -> Result<()> {
    match mkdir(PROJECT_DIR_NAME) {
        Ok(()) => {
            let conn = try!(new_conn());

            migrations::run(&conn)
        },
        Err(code) => Err(Error::libc(code)),
    }
}

pub fn init() {
    match run_init() {
        Ok(()) => exit("Initialized science project in .science directory.", 0),
        Err(err) => exit(err, 1),
    }
}

fn run_start(description: &str, status: &str) -> Result<(Experiment, Datapoint)> {
    let owned_description = String::from(description);
    let owned_status = String::from(status);
    let mut conn = try!(new_conn());
    let opt_session = try!(Experiment::current(&conn));

    match opt_session {
        Some(_) => Err(Error::other(String::from("A science experiment is already in progress.  To record a new datapoint, run `science record`."))),
        None => {
            let tx = try_sqlite!(conn.transaction());
            let session = try!(Experiment::create(&tx));

            try!(session.make_current(&tx));

            let sha = try!(lookup_git_sha());
            let point = try!(Datapoint::create(&tx, &owned_description, session.id, &sha, &owned_status));

            try_sqlite!(tx.commit());

            Ok((session, point))
        },
    }
}

pub fn start(description: &str, status: &str) {
    match run_start(description, status) {
        Ok((_, _)) => exit("Started experiment.", 0),
        Err(err) => exit(err, 1),
    };
}

fn run_record(description: &str, status: &str) -> Result<Datapoint> {
    let owned_description = String::from(description);
    let owned_status = String::from(status);
    let conn = try!(new_conn());
    let opt_session = try!(Experiment::current(&conn));

    match opt_session {
        Some(session) => {
            match git_commit(description, status) {
                // Once we've successfully committed, rollback is tricky.  We don't attempt to
                // persist the datapoint again, since having a DB error means that there is an
                // increased likelihood of another DB error when retrying.  Instead, the approach
                // is to give comprehensive instructions on how to fix the state.
                Ok(()) => match lookup_git_sha() {
                    Ok(sha) => match Datapoint::create(&conn, &owned_description, session.id, &sha, &owned_status) {
                        Ok(datapoint) => Ok(datapoint),
                        // Just persisting the datapoint failed, so we tell the user to run
                        // something like:
                        //
                        // (cd .science && sqlite3 Science.db)
                        // sqlite> INSERT INTO datapoints (description, session_id, sha, status) VALUES ('Changed something', 1, '4ab439dcf00', 'failing');
                        Err(Error(msg)) => Err(Error(format!("{}\n\nScience was able to commit the changes, but the datapoint was not persisted to the .science directory.  To fix the issue, try running\n\n(cd .science && sqlite3 Science.db)\nINSERT INTO datapoints (description, session_id, sha, status) VALUES ('{}', {}, '{}', '{}')", msg, description, session.id, sha, status))),
                    },
                    // Looking up the git SHA failed, so we tell the user to run something like:
                    //
                    // git rev-parse HEAD
                    // (cd .science && sqlite3 Science.db)
                    // sqlite> INSERT INTO datapoints (description, session_id, sha, status) VALUES ('Changed something', 1, '<insert SHA here>', 'failing');
                    Err(Error(msg)) => Err(Error(format!("{}\n\nScience was able to commit the changes, but could not look up the SHA of the resulting commit, so the datapoint was not persisted to the .science directory.  To fix the issue, try running\n\ngit rev-parse HEAD # Gets the SHA of the commit\n(cd .science && sqlite3 Science.db)\nINSERT INTO datapoints (description, session_id, sha, status) VALUES ('{}', {}, '<insert SHA here>', '{}');", msg, description, session.id, status))),
                },
                // If we error during the commit we can just pass it through to the user, as there
                // is no cleanup to do.
                Err(err) => Err(err),
            }
        },
        None => Err(Error(String::from("You need to start a science experiment first.  Run `science start`."))),
    }
}

pub fn record(description: &str, status: &str) {
    match run_record(description, status) {
        Ok(_) => exit("Recorded datapoint.", 0),
        Err(err) => exit(err, 1),
    }
}

fn run_stop() -> Result<()> {
    let mut conn = try!(new_conn());
    let opt_session = try!(Experiment::current(&conn));

    match opt_session {
        Some(session) => {
            // We create a transaction here, as there's a chance that the session will need to be
            // deleted from two places, the sessions table and the current_session table.
            let tx = try_sqlite!(conn.transaction());

            try!(session.delete(&tx));

            try_sqlite!(tx.commit());

            Ok(())
        },
        None => Err(Error(String::from("There is no ongoing science experiment to stop."))),
    }
}

pub fn stop() {
    match run_stop() {
        Ok(()) => exit("Stopped experiment.", 0),
        Err(err) => exit(err, 1),
    }
}

pub fn analyze() {
}
