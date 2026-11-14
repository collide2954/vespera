use std::io;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(60);

fn run_shard(address: &str) -> io::Result<Output> {
    let child = Command::new(env!("CARGO_BIN_EXE_vespera-server"))
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .env("VESPERA_SHARD_ADDRESS", address)
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
fn the_shard_names_the_cause_when_its_port_is_taken() -> io::Result<()> {
    let taken = UdpSocket::bind("127.0.0.1:0")?;

    let output = run_shard(&taken.local_addr()?.to_string())?;

    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        errors.contains("error: cannot open the UDP socket: "),
        "{errors}"
    );
    Ok(())
}

#[test]
fn the_shard_names_a_bad_address_variable() -> io::Result<()> {
    let output = run_shard("not an address")?;

    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        errors.contains("error: VESPERA_SHARD_ADDRESS is not a socket address: "),
        "{errors}"
    );
    Ok(())
}
