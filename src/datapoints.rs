use rusqlite::Connection;
use util::{Error, Result};

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
