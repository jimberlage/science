use rusqlite::{self, Connection, Statement, Transaction};
use util::{Error, Result};

// This is temporarily necessary, as rusqlite doesn't have a way to genericize over Connections and
// Transactions, which have lots of shared functionality.
//
// TODO: Add this to rusqlite.
pub trait ConnectionLike {
    fn my_prepare<'a>(&'a self, sql: &str) -> rusqlite::Result<Statement<'a>>;

    fn my_last_insert_rowid(&self) -> i64;
}

impl ConnectionLike for Connection {
    fn my_prepare<'a>(&'a self, sql: &str) -> rusqlite::Result<Statement<'a>> {
        self.prepare(sql)
    }

    fn my_last_insert_rowid(&self) -> i64 {
        self.last_insert_rowid()
    }
}

impl<'b> ConnectionLike for Transaction<'b> {
    fn my_prepare<'a>(&'a self, sql: &str) -> rusqlite::Result<Statement<'a>> {
        self.prepare(sql)
    }

    fn my_last_insert_rowid(&self) -> i64 {
        self.last_insert_rowid()
    }
}

pub struct Experiment {
    pub id: i64
}

impl Experiment {
    pub fn create<T>(conn: &T) -> Result<Experiment> where T: ConnectionLike {
        let mut stmt = try_sqlite!(conn.my_prepare("INSERT INTO sessions DEFAULT VALUES"));

        match stmt.execute(&[]) {
            Ok(_) => Ok(Experiment { id: conn.my_last_insert_rowid() }),
            Err(err) => Err(Error::sqlite(err)),
        }
    }

    pub fn current<T>(conn: &T) -> Result<Option<Experiment>> where T: ConnectionLike {
        match conn.my_prepare("SELECT id FROM current_session LIMIT 1") {
            Ok(mut stmt) => {
                let mut sessions = try_sqlite!(stmt.query_map(&[], |row| {
                    Experiment { id: row.get("id") }
                }));

                match sessions.next() {
                    Some(session) => match session {
                        Ok(session) => Ok(Some(session)),
                        Err(err) => Err(Error::sqlite(err)),
                    },
                    None => Ok(None),
                }
            },
            Err(err) => Err(Error::sqlite(err)),
        }
    }

    pub fn make_current<T>(&self, conn: &T) -> Result<()> where T: ConnectionLike {
        let mut stmt = try_sqlite!(conn.my_prepare("INSERT INTO current_session (id) VALUES (?)"));

        match stmt.execute(&[&self.id]) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::sqlite(err)),
        }
    }

    fn simple_delete<T>(&self, conn: &T) -> Result<()> where T: ConnectionLike {
        let mut stmt = try_sqlite!(conn.my_prepare("DELETE FROM sessions WHERE id = ?"));

        match stmt.execute(&[&self.id]) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::sqlite(err)),
        }
    }

    pub fn delete(&self, conn: &Transaction) -> Result<()> {
        match Experiment::current(conn) {
            Ok(Some(ref session)) if (*session).id == self.id => {
                try!(self.simple_delete(conn));

                let mut stmt = try_sqlite!(conn.my_prepare("DELETE FROM current_session"));

                match stmt.execute(&[]) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(Error::sqlite(err)),
                }
            },
            Ok(_) => self.simple_delete(conn),
            Err(err) => Err(err)
        }
    }
}

pub struct Datapoint {
    id: i64,
    description: String,
    session_id: i64,
    sha: String,
    status: String,
}

impl Datapoint {
    pub fn create<T>(conn: &T,
                     description: &String,
                     session_id: i64,
                     sha: &String,
                     status: &String) -> Result<Datapoint> where T: ConnectionLike {

        let mut stmt = try_sqlite!(conn.my_prepare("INSERT INTO datapoints (description, session_id, sha, status) VALUES (?, ?, ?, ?)"));

        match stmt.execute(&[description, &session_id, sha, status]) {
            Ok(_) => Ok(Datapoint {
                id: conn.my_last_insert_rowid(),
                description: description.clone(),
                session_id: session_id,
                sha: sha.clone(),
                status: status.clone(),
            }),
            Err(err) => Err(Error::sqlite(err)),
        }
    }
}
