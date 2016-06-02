use datapoints::{self, Datapoint};
use migrations;
use sessions::{self, Session};
use util::{git_commit, lookup_git_sha, mkdir, new_conn, Error, PROJECT_DIR_NAME, Result};

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

    // TODO: Revise this to handle a rollback.
    match opt_session {
        Some(_) => Err(Error::other(String::from("A science experiment is already in progress.  To record a new datapoint, run `science record`."))),
        None => {
            let session = try!(sessions::create(&conn));

            try!(session.make_current(&conn));

            let sha = try!(lookup_git_sha());
            let point = try!(datapoints::create(&conn, &owned_description, session.id, &sha, &owned_status));

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
            match git_commit(description, status) {
                // Once we've successfully committed, rollback is tricky.  We don't attempt to
                // persist the datapoint again, since having a DB error means that there is an
                // increased likelihood of another DB error when retrying.  Instead, the approach
                // is to give comprehensive instructions on how to fix the state.
                Ok(()) => match lookup_git_sha() {
                    Ok(sha) => match datapoints::create(&conn, &owned_description, session.id, &sha, &owned_status) {
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

pub fn stop() -> Result<()> {
    Ok(())
}
