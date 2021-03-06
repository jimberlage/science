use rusqlite::Connection;
use std::collections::HashSet;
use std::os::raw::c_int;
use util::Result;

/* Database migrations are executed in the order they are read.  To add a new migration, add the
 * necessary SQL here.
 * */
const MIGRATIONS: [&'static str; 4] = [
    "CREATE TABLE migrations(id INTEGER)",
    "CREATE TABLE experiments(id INTEGER PRIMARY KEY)",
    "CREATE TABLE datapoints(id INTEGER PRIMARY KEY NOT NULL, experiment_id INTEGER NOT NULL, sha VARCHAR(255) NOT NULL, description TEXT NOT NULL, status VARCHAR(255) NOT NULL)",
    "CREATE TABLE current_experiment(id INTEGER)",
];

/* select_migrations looks in the `migrations` table in the DB, and returns all the migrations
 * we have already run.
 *
 * This function assumes that a `migrations` table already exists in the DB.
 * */
fn select_migrations(conn: &Connection) -> Result<HashSet<c_int>> {
    let mut stmt = try_generic!(conn.prepare("SELECT id FROM migrations"));

    let ids = try_generic!(stmt.query_map(&[], |row| { row.get("id") }));
    let mut result = HashSet::new();

    for id in ids {
        result.insert(try_generic!(id));
    }

    Ok(result)
}

/* finished_migrations returns all the migrations we have already run, but does not assume that
 * there is a migrations table already.
 * */
fn finished_migrations(conn: &Connection) -> Result<HashSet<c_int>> {
    let mut stmt = try_generic!(conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'migrations' LIMIT 1"));
    let rows = try_generic!(stmt.query_map(&[], |_| { true }));

    if rows.count() == 0 {
        // If there's no migrations table, we assume we have to run all the migrations.
        Ok(HashSet::new())
    } else {
        // Otherwise, we'll just make a set of the migrations we've already run.
        select_migrations(&conn)
    }
}

pub fn run(conn: &Connection) -> Result<()> {
    let finished = try_generic!(finished_migrations(conn));
    let mut to_run = vec![];

    for i in 0..MIGRATIONS.len() {
        if !finished.contains(&(i as c_int)) {
            to_run.push(String::from(MIGRATIONS[i]));
            // Since i isn't controlled by the user, we won't bother using a prepared
            // statement.
            to_run.push(format!("INSERT INTO migrations (id) VALUES ({})", i));
        }
    }

    // Ensure that we only hit the DB if we actually have migrations to run.
    if to_run.len() > 0 {
        let sql = format!("BEGIN;\n{};\nCOMMIT;", to_run.join(";\n"));

        Ok(try_generic!(conn.execute_batch(sql.as_str())))
    } else {
        Ok(())
    }
}
