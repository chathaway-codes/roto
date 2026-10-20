#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    match cargo_roto::run_from_env() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
