extern crate rusqlite;

use libc::{self, EEXIST, S_IRWXU, S_IRGRP, S_IXGRP, S_IROTH, S_IXOTH};
use rusqlite::Connection;
use std::ffi::CString;
use std::os::raw::c_int;
use std::process::Command;
use std::result;

pub const PROJECT_DIR_NAME: &'static str = ".science";
pub const PROJECT_DB_NAME: &'static str = "Science.db";

pub struct Error(pub String);

pub fn libc_error(code: c_int) -> Error {
    Error(format!("Libc error: {}", code))
}

pub fn sqlite_error(msg: rusqlite::Error) -> Error {
    Error(format!("SQLite error: {}", msg))
}

pub type Result<T> = result::Result<T, Error>;

pub fn mkdir(dir: &str) -> result::Result<(), c_int> {
    unsafe {
        let filename = CString::new(String::from(dir)).unwrap().into_raw();
        // drwxr-xr-x
        // http://www.gnu.org/software/libc/manual/html_mono/libc.html#Permission-Bits
        let permissions = S_IRWXU | S_IRGRP | S_IXGRP | S_IROTH | S_IXOTH;
        let result = match libc::mkdir(filename, permissions) {
            // We're okay with some failures, since it's all right if the directory already
            // exists.
            -1 | 0 | EEXIST => Ok(()),
            // If we get a weird code, error.
            code => Err(code),
        };

        // Put the pointer back into a rust-owned data type, so rust will deallocate it.
        CString::from_raw(filename);

        result
    }
}

pub fn new_conn() -> Result<Connection> {
    match Connection::open(format!("{}/{}", PROJECT_DIR_NAME, PROJECT_DB_NAME)) {
        Ok(conn) => Ok(conn),
        Err(err) => Err(sqlite_error(err)),
    }
}

// Creates a git commit with the given description and status.
pub fn git_commit(description: &str, status: &str) -> Result<()> {
    let error = Error(String::from("`git commit` failed."));
    let commit_msg = format!("(science commit)\n\ndescription:\n\n{}\n\nstatus:\n\n{}", description, status);

    match Command::new("git").arg("commit").arg("-m").arg(commit_msg).output() {
        Ok(output) => if output.status.success() {
            Ok(())
        } else {
            match String::from_utf8(output.stdout) {
                Ok(stdout) => Err(Error(stdout)),
                Err(_) => Err(error),
            }
        },
        Err(_) => Err(error),
    }
}

pub fn lookup_git_sha() -> Result<String> {
    let error = Error(String::from("`git rev-parse HEAD` failed."));

    match Command::new("git").arg("rev-parse").arg("HEAD").output() {
        Ok(output) => if output.status.success() {
            match String::from_utf8(output.stdout) {
                Ok(sha) => Ok(sha),
                Err(_) => Err(Error(String::from("`git rev-parse HEAD` returned invalid ASCII/UTF-8 output."))),
            }
        } else {
            Err(error)
        },
        Err(_) => Err(error),
    }
}