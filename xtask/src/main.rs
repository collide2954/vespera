mod command;
mod comments;
mod finding;
mod repository;
mod text_only;

use std::process::ExitCode;

fn main() -> ExitCode {
    match command::run(std::env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
