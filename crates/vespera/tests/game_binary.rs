use std::io;
use std::process::Command;

#[test]
fn the_game_names_a_bad_server_variable() -> io::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_vespera"))
        .env("VESPERA_SERVER", "not an address")
        .output()?;

    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        errors.contains("error: VESPERA_SERVER is not a socket address: "),
        "{errors}"
    );
    Ok(())
}
