use datapoints::{self, Datapoint};
use migrations;
use sessions::{self, Session};
use util::{libc_error, mkdir, new_conn, Error, PROJECT_DIR_NAME, Result};

pub fn init() -> Result<()> {
    match mkdir(PROJECT_DIR_NAME) {
        Ok(()) => {
            let conn = try!(new_conn());

            migrations::run(&conn)
        },
        Err(code) => Err(libc_error(code)),
    }
}

pub fn start(description: &str, status: &str) -> Result<(Session, Datapoint)> {
    let owned_description = String::from(description);
    let owned_status = String::from(status);
    let conn = try!(new_conn());
    let opt_session = try!(sessions::current(&conn));

    match opt_session {
        Some(_) => Err(Error(String::from("A science experiment is already in progress.  To record a new datapoint, run `science record`."))),
        None => {
            let session = try!(sessions::create(&conn));

            try!(session.make_current(&conn));

            let point = try!(datapoints::record(&conn, &session, &owned_description, &owned_status, true));

            Ok((session, point))
        },
    }
}

pub fn record(description: &str, status: &str) -> Result<Datapoint> {
    let owned_description = String::from(description);
    let owned_status = String::from(status);
    let conn = try!(new_conn());
    let opt_session = try!(sessions::current(&conn));

    match opt_session {
        Some(session) => {
            let point = try!(datapoints::record(&conn, &session, &owned_description, &owned_status, true));

            Ok(point)
        },
        None => Err(Error(String::from("You need to start a science experiment first.  Run `science start`."))),
    }
}
