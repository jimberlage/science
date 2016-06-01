use rusqlite::Connection;
use sessions::Session;
use util::{git_commit, lookup_git_sha, Error, Result};

pub struct Datapoint {
    id: i64,
    description: String,
    session_id: i64,
    sha: String,
    status: String,
}

pub fn create(conn: &Connection,
              description: &String,
              session_id: i64,
              sha: &String,
              status: &String) -> Result<Datapoint> {

    match conn.prepare("INSERT INTO datapoints (description, session_id, sha, status) VALUES (?, ?, ?, ?)") {
        Ok(mut stmt) => match stmt.execute(&[description, &session_id, sha, status]) {
            Ok(_) => Ok(Datapoint {
                id: conn.last_insert_rowid(),
                description: description.clone(),
                session_id: session_id,
                sha: sha.clone(),
                status: status.clone(),
            }),
            Err(err) => Err(Error::sqlite(err)),
        },
        Err(err) => Err(Error::sqlite(err)),
    }
}

pub fn record(conn: &Connection,
              session: &Session,
              description: &String,
              status: &String,
              commit: bool) -> Result<Datapoint> {

    if commit {
        try!(git_commit(description, status));
    }

    let sha = try!(lookup_git_sha());

    create(conn, description, session.id, &sha, status)
}
