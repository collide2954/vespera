mod assets;
mod bench;
mod command;
mod comments;
mod docs;
mod dxc;
mod finding;
mod layers;
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
