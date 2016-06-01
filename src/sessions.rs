extern crate rusqlite;

use rusqlite::Connection;
use util::{sqlite_error, Result};

pub struct Session {
    pub id: i64
}

impl Session {
    pub fn make_current(&self, conn: &Connection) -> Result<()> {
        match conn.prepare("INSERT INTO current_session (id) VALUES (?)") {
            Ok(mut stmt) => match stmt.execute(&[&self.id]) {
                Ok(_) => Ok(()),
                Err(err) => Err(sqlite_error(err)),
            },
            Err(err) => Err(sqlite_error(err)),
        }
    }
}

pub fn create(conn: &Connection) -> Result<Session> {
    match conn.prepare("INSERT INTO sessions DEFAULT VALUES") {
        Ok(mut stmt) => match stmt.execute(&[]) {
            Ok(_) => Ok(Session { id: conn.last_insert_rowid() }),
            Err(err) => Err(sqlite_error(err)),
        },
        Err(err) => Err(sqlite_error(err)),
    }
}

pub fn current(conn: &Connection) -> Result<Option<Session>> {
    match conn.prepare("SELECT id FROM current_session LIMIT 1") {
        Ok(mut stmt) => match stmt.query_map(&[], |row| { Session { id: row.get("id") } }) {
            Ok(mut sessions) => match sessions.next() {
                Some(session) => match session {
                    Ok(session) => Ok(Some(session)),
                    Err(err) => Err(sqlite_error(err)),
                },
                None => Ok(None),
            },
            Err(err) => Err(sqlite_error(err)),
        },
        Err(err) => Err(sqlite_error(err)),
    }
}
