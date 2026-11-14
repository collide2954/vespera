use std::io;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(60);

fn run_game(variable: &str, value: &str) -> io::Result<Output> {
    let child = Command::new(env!("CARGO_BIN_EXE_vespera"))
        .env(variable, value)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    finish_or_stop(child)
}

fn finish_or_stop(mut child: Child) -> io::Result<Output> {
    let start = Instant::now();
    while child.try_wait()?.is_none() {
        if start.elapsed() > TIMEOUT {
            child.kill()?;
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    child.wait_with_output()
}

#[test]
fn the_game_names_a_bad_server_variable() -> io::Result<()> {
    let output = run_game("VESPERA_SERVER", "not an address")?;

    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        errors.contains("error: VESPERA_SERVER is not a socket address: "),
        "{errors}"
    );
    Ok(())
}

#[test]
fn the_game_names_a_bad_benchmark_variable() -> io::Result<()> {
    let output = run_game("VESPERA_BENCHMARK_SECONDS", "soon")?;

    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        errors.contains("error: VESPERA_BENCHMARK_SECONDS is not a whole number of seconds: "),
        "{errors}"
    );
    Ok(())
}
