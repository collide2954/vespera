use std::io;
use std::process::{Command, Output};

fn xtask(arguments: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(arguments)
        .output()
}

#[test]
fn check_text_only_passes_on_this_repository() -> io::Result<()> {
    let output = xtask(&["check-text-only"])?;

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(())
}

#[test]
fn check_comments_passes_on_this_repository() -> io::Result<()> {
    let output = xtask(&["check-comments"])?;

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(())
}

#[test]
fn check_docs_passes_on_this_repository() -> io::Result<()> {
    let output = xtask(&["check-docs"])?;

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(())
}

#[test]
fn check_layers_passes_on_this_repository() -> io::Result<()> {
    let output = xtask(&["check-layers"])?;

    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn an_unknown_command_fails_with_a_message() -> io::Result<()> {
    let output = xtask(&["fly"])?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown command: fly"));
    Ok(())
}
