use rusqlite::Connection;
use std::fmt::{self, Display, Formatter};
use std::fs::{DirBuilder, OpenOptions};
use std::io::Write;
use std::process::Command;
use std::result;

pub const PROJECT_DIR: &'static str = ".science";
pub const PROJECT_DB_FILE: &'static str = "Science.db";
pub const PROJECT_LOG_FILE: &'static str = "client.log";

pub fn logfile_path() -> String {
    format!("{}/{}", PROJECT_DIR, PROJECT_LOG_FILE)
}

pub fn db_path() -> String {
    format!("{}/{}", PROJECT_DIR, PROJECT_DB_FILE)
}

pub enum Error {
    Generic(Option<String>),
    Specific(Option<String>, String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> result::Result<(), fmt::Error> {
        match self {
            &Error::Generic(_) => write!(f, "An error occurred.  Look in {} for more details.", logfile_path()),
            &Error::Specific(_, ref msg) => write!(f, "{}", msg),
        }
    }
}

pub fn log_string(msg: &String) -> Result<()> {
    match OpenOptions::new().append(true).create(true).open(&logfile_path()) {
        Ok(mut file) => match write!(file, "{}\n", msg) {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::Generic(None)),
        },
        Err(_) => Err(Error::Generic(None)),
    }
}

pub fn log(err: &Error) -> Result<()> {
    match err {
        &Error::Generic(None) => Ok(()),
        &Error::Generic(Some(ref msg)) => log_string(msg),
        &Error::Specific(None, _) => Ok(()),
        &Error::Specific(Some(ref msg), _) => log_string(msg),
    }
}

pub fn specific_error<T>(err: Option<T>, msg: String) -> Error where T: Display {
    match err {
        Some(err) => Error::Specific(Some(format!("{}", err)), msg),
        None => Error::Specific(None, msg),
    }
}

pub fn generic_error<T>(err: T) -> Error where T: Display {
    Error::Generic(Some(format!("{}", err)))
}

// Like try!, but converts an Error to a Error::Generic.
#[macro_export]
macro_rules! try_generic {
    ($expr:expr) => (match $expr {
        Ok(val) => val,
        Err(err) => return Err($crate::util::generic_error(err)),
    })
}

#[macro_export]
macro_rules! try_and_log {
    ($expr:expr) => (match $expr {
        Ok(val) => val,
        Err(err) => match $crate::util::log(&err) {
            Ok(()) => return Err(err),
            Err(_) => panic!("Encountered an error while handling another error."),
        },
    })
}

#[macro_export]
macro_rules! try_generic_and_log {
    ($expr:expr) => (match $expr {
        Ok(val) => val,
        Err(err) => {
            let generic = $crate::util::generic_error(err);

            match $crate::util::log(&generic) {
                Ok(()) => return Err(generic),
                Err(_) => panic!("Encountered an error while handling another error."),
            }
        },
    })
}

pub type Result<T> = result::Result<T, Error>;

pub fn mkdir(dir: &str) -> Result<()> {
    match DirBuilder::new().create(dir) {
        Ok(()) => Ok(()),
        Err(err) => Err(generic_error(err)),
    }
}

pub fn new_conn() -> Result<Connection> {
    match Connection::open(db_path()) {
        Ok(conn) => Ok(conn),
        Err(err) => Err(generic_error(err)),
    }
}

// Creates a git commit with the given description and status.
pub fn git_commit(description: &str, status: &str) -> Result<()> {
    let commit_msg = format!("(science commit)\n\ndescription:\n\n{}\n\nstatus:\n\n{}", description, status);
    let output = try_generic!(Command::new("git").arg("commit").arg("-m").arg(commit_msg).output());

    if output.status.success() {
        Ok(())
    } else {
        let error: Option<String> = None;
        match String::from_utf8(output.stdout) {
            Ok(stdout) => Err(specific_error(error, stdout)),
            Err(err) => Err(generic_error(err)),
        }
    }
}

pub fn lookup_git_sha() -> Result<String> {
    let output = try_generic!(Command::new("git").arg("rev-parse").arg("HEAD").output());

    if output.status.success() {
        match String::from_utf8(output.stdout) {
            Ok(sha) => Ok(sha),
            Err(err) => Err(generic_error(err)),
        }
    } else {
        Err(Error::Generic(None))
    }
}
