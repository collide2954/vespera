use std::env::consts::EXE_SUFFIX;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};

const READY_LINE: &str = "the shard listens on";
const SHARD_START_TIME: Duration = Duration::from_secs(30);

pub fn run(root: &Path, seconds: u64) -> Result<()> {
    let (shard_binary, client_binary) = binaries(root)?;
    let mut shard = Command::new(&shard_binary)
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("cannot start {}", shard_binary.display()))?;
    let result = benchmark_with_shard(&mut shard, root, &client_binary, seconds);
    if let Err(error) = shard.kill() {
        println!("the shard stopped before the end of the benchmark: {error}");
    }
    let _status = shard.wait();
    result
}

fn binaries(root: &Path) -> Result<(PathBuf, PathBuf)> {
    let folder = root.join("target").join("debug");
    let shard = folder.join(format!("vespera-server{EXE_SUFFIX}"));
    let client = folder.join(format!("vespera{EXE_SUFFIX}"));
    for binary in [&shard, &client] {
        ensure!(
            binary.is_file(),
            "{} does not exist. Run cargo build --workspace first",
            binary.display()
        );
    }
    Ok((shard, client))
}

fn benchmark_with_shard(shard: &mut Child, root: &Path, client: &Path, seconds: u64) -> Result<()> {
    let log = shard.stderr.take().context("the shard has no log stream")?;
    let (ready, is_ready) = mpsc::channel();
    thread::spawn(move || watch_shard_log(BufReader::new(log), &ready));
    match is_ready.recv_timeout(SHARD_START_TIME) {
        Ok(()) => {}
        Err(RecvTimeoutError::Timeout) => bail!(
            "the shard did not listen after {} s",
            SHARD_START_TIME.as_secs()
        ),
        Err(RecvTimeoutError::Disconnected) => bail!(
            "the shard stopped at the start. Start target/debug/vespera-server alone to see the error"
        ),
    }
    println!("the shard listens. The client records frames for {seconds} s");
    let status = Command::new(client)
        .current_dir(root)
        .env("VESPERA_BENCHMARK_SECONDS", seconds.to_string())
        .status()
        .with_context(|| format!("cannot start {}", client.display()))?;
    ensure!(status.success(), "the client stopped with {status}");
    Ok(())
}

fn watch_shard_log(log: impl BufRead, ready: &Sender<()>) {
    for line in log.lines().map_while(std::result::Result::ok) {
        if line.contains(READY_LINE) {
            let _ = ready.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Cursor;
    use std::sync::mpsc;

    use super::{binaries, watch_shard_log};

    #[test]
    fn missing_binaries_are_an_error_that_names_the_build_command() {
        let root = tempfile::tempdir().unwrap();

        let error = binaries(root.path()).unwrap_err().to_string();

        assert!(error.contains("cargo build --workspace"), "{error}");
    }

    #[test]
    fn the_binaries_are_in_the_debug_folder() {
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("target").join("debug");
        fs::create_dir_all(&folder).unwrap();
        let suffix = std::env::consts::EXE_SUFFIX;
        fs::write(folder.join(format!("vespera-server{suffix}")), "").unwrap();
        fs::write(folder.join(format!("vespera{suffix}")), "").unwrap();

        let (shard, client) = binaries(root.path()).unwrap();

        assert_eq!(shard, folder.join(format!("vespera-server{suffix}")));
        assert_eq!(client, folder.join(format!("vespera{suffix}")));
    }

    #[test]
    fn the_shard_is_ready_when_its_log_shows_the_address() {
        let log = "loading content\n2026-11-15T10:42:51Z INFO vespera_server: the shard listens on 127.0.0.1:5000\nmore\n";
        let (ready, is_ready) = mpsc::channel();

        watch_shard_log(Cursor::new(log), &ready);

        assert_eq!(is_ready.try_recv(), Ok(()));
    }

    #[test]
    fn a_shard_log_without_the_address_gives_no_signal() {
        let (ready, is_ready) = mpsc::channel();

        watch_shard_log(Cursor::new("error: the port is taken\n"), &ready);
        drop(ready);

        assert!(is_ready.recv().is_err());
    }
}
