use rusqlite::{self, Connection, Statement, Transaction};
use util::{generic_error, Result};

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
        let mut stmt = try_generic!(conn.my_prepare("INSERT INTO experiments DEFAULT VALUES"));

        match stmt.execute(&[]) {
            Ok(_) => Ok(Experiment { id: conn.my_last_insert_rowid() }),
            Err(err) => Err(generic_error(err)),
        }
    }

    pub fn current<T>(conn: &T) -> Result<Option<Experiment>> where T: ConnectionLike {
        let mut stmt = try_generic!(conn.my_prepare("SELECT id FROM current_experiment LIMIT 1"));
        let mut experiments = try_generic!(stmt.query_map(&[], |row| {
            Experiment { id: row.get("id") }
        }));

        match experiments.next() {
            Some(experiment) => match experiment {
                Ok(experiment) => Ok(Some(experiment)),
                Err(err) => Err(generic_error(err)),
            },
            None => Ok(None),
        }
    }

    pub fn delete_current<T>(conn: &T) -> Result<()> where T: ConnectionLike {
        let mut stmt = try_generic!(conn.my_prepare("DELETE FROM current_experiment"));

        match stmt.execute(&[]) {
            Ok(_) => Ok(()),
            Err(err) => Err(generic_error(err)),
        }
    }

    pub fn make_current<T>(&self, conn: &T) -> Result<()> where T: ConnectionLike {
        let mut stmt = try_generic!(conn.my_prepare("INSERT INTO current_experiment (id) VALUES (?)"));

        match stmt.execute(&[&self.id]) {
            Ok(_) => Ok(()),
            Err(err) => Err(generic_error(err)),
        }
    }
}

pub struct Datapoint {
    id: i64,
    description: String,
    experiment_id: i64,
    sha: String,
    status: String,
}

impl Datapoint {
    pub fn create<T>(conn: &T,
                     description: &String,
                     experiment_id: i64,
                     sha: &String,
                     status: &String) -> Result<Datapoint> where T: ConnectionLike {

        let mut stmt = try_generic!(conn.my_prepare("INSERT INTO datapoints (description, experiment_id, sha, status) VALUES (?, ?, ?, ?)"));

        match stmt.execute(&[description, &experiment_id, sha, status]) {
            Ok(_) => Ok(Datapoint {
                id: conn.my_last_insert_rowid(),
                description: description.clone(),
                experiment_id: experiment_id,
                sha: sha.clone(),
                status: status.clone(),
            }),
            Err(err) => Err(generic_error(err)),
        }
    }
}
