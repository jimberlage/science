use datapoints::{self, Datapoint};
use migrations;
use sessions::{self, Session};
use util::{mkdir, new_conn, Error, GitOperation, PROJECT_DIR_NAME, Result};

pub fn init() -> Result<()> {
    match mkdir(PROJECT_DIR_NAME) {
        Ok(()) => {
            let conn = try!(new_conn());

            migrations::run(&conn)
        },
        Err(code) => Err(Error::libc(code)),
    }
}

pub fn start(description: &str, status: &str) -> Result<(Session, Datapoint)> {
    let owned_description = String::from(description);
    let owned_status = String::from(status);
    let conn = try!(new_conn());
    let opt_session = try!(sessions::current(&conn));

    match opt_session {
        Some(_) => Err(Error::other(String::from("A science experiment is already in progress.  To record a new datapoint, run `science record`."))),
        None => {
            let session = try!(sessions::create(&conn));

            try!(session.make_current(&conn));

            let point = try!(datapoints::record(&conn, &session, &owned_description, &owned_status, false));

            Ok((session, point))
        },
    }
}

// TODO: Refactor this.  Not really happy with it.
fn unrecoverable_msg(error_msg: String) -> String {
    format!("{}\n\nThis datapoint was given a git commit, but a subsequent call failed and the datapoint was not persisted in the .science directory.  Your git history is fine, but this datapoint will not show up when you run `science analyze`.", error_msg)
}

pub fn record(description: &str, status: &str) -> Result<Datapoint> {
    let owned_description = String::from(description);
    let owned_status = String::from(status);
    let conn = try!(new_conn());
    let opt_session = try!(sessions::current(&conn));

    match opt_session {
        Some(session) => {
            match datapoints::record(&conn, &session, &owned_description, &owned_status, true) {
                Ok(point) => Ok(point),
                Err(err) => Err(Error::other(unrecoverable_msg(err.to_string()))),
            }
        },
        None => Err(Error::other(String::from("You need to start a science experiment first.  Run `science start`."))),
    }
}
