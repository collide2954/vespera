use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};

pub const VERSION: (u32, u32) = (5, 3);

const VARIABLE: &str = "BLENDER";
const TAIL_LINES: usize = 12;

pub trait Blender {
    fn run(&self, arguments: &[OsString]) -> Result<String>;
}

pub struct Program(pub PathBuf);

impl Program {
    pub fn from_environment() -> Self {
        Self(std::env::var_os(VARIABLE).map_or_else(|| PathBuf::from("blender"), PathBuf::from))
    }
}

impl Blender for Program {
    fn run(&self, arguments: &[OsString]) -> Result<String> {
        let output = Command::new(&self.0).args(arguments).output().with_context(|| {
            format!(
                "cannot start Blender as {}. Install Blender {}.{}, or set the {VARIABLE} variable to the Blender program",
                self.0.display(),
                VERSION.0,
                VERSION.1
            )
        })?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        ensure!(
            output.status.success(),
            "Blender stopped with {}:\n{}",
            output.status,
            tail(&String::from_utf8_lossy(&output.stderr), &stdout)
        );
        Ok(stdout)
    }
}

pub fn check_version(blender: &impl Blender) -> Result<String> {
    let output = blender.run(&[OsString::from("--version")])?;
    let (text, numbers) = output
        .lines()
        .find_map(version_in)
        .context("Blender did not show its version")?;
    ensure!(
        numbers == VERSION,
        "the import step uses Blender {}.{}, but the Blender program has the version {text}",
        VERSION.0,
        VERSION.1
    );
    Ok(text.to_owned())
}

pub fn script_arguments(script: &Path, arguments: &[&Path]) -> Vec<OsString> {
    let mut all: Vec<OsString> = ["--background", "--python-exit-code", "1", "--python"]
        .map(OsString::from)
        .into();
    all.push(script.as_os_str().to_owned());
    all.push(OsString::from("--"));
    all.extend(arguments.iter().map(|path| path.as_os_str().to_owned()));
    all
}

fn version_in(line: &str) -> Option<(&str, (u32, u32))> {
    let text = line
        .trim()
        .strip_prefix("Blender ")?
        .split_whitespace()
        .next()?;
    let mut numbers = text.split('.').map(str::parse::<u32>);
    let major = numbers.next()?.ok()?;
    let minor = numbers.next()?.ok()?;
    Some((text, (major, minor)))
}

fn tail(stderr: &str, stdout: &str) -> String {
    let text = if stderr.trim().is_empty() {
        stdout
    } else {
        stderr
    };
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    lines[lines.len().saturating_sub(TAIL_LINES)..].join("\n")
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use anyhow::Result;

    use super::{Blender, Program, check_version, script_arguments};

    struct Prints(&'static str);

    impl Blender for Prints {
        fn run(&self, _arguments: &[OsString]) -> Result<String> {
            Ok(self.0.to_owned())
        }
    }

    #[test]
    fn blender_5_3_passes_the_version_check() {
        let output = "Blender 5.3.0\n\tbuild date: 2026-11-10\n\tbuild hash: 1a2b3c\n";

        assert_eq!(check_version(&Prints(output)).unwrap(), "5.3.0");
    }

    #[test]
    fn a_version_line_after_other_lines_is_found() {
        let output = "Color management: using fallback mode\nBlender 5.3.2 LTS\n";

        assert_eq!(check_version(&Prints(output)).unwrap(), "5.3.2");
    }

    #[test]
    fn a_different_blender_version_is_an_error_with_both_versions() {
        let message = format!(
            "{:#}",
            check_version(&Prints("Blender 4.2.3\n")).unwrap_err()
        );

        assert!(message.contains("4.2.3"), "{message}");
        assert!(message.contains("5.3"), "{message}");
    }

    #[test]
    fn an_output_without_a_version_is_an_error() {
        assert!(check_version(&Prints("command not found\n")).is_err());
    }

    #[test]
    fn a_script_runs_in_the_background_with_its_arguments_after_two_dashes() {
        let arguments = script_arguments(
            Path::new("xtask/blender/human.py"),
            &[Path::new("hero.json"), Path::new("hero.blend")],
        );

        assert_eq!(
            arguments,
            [
                "--background",
                "--python-exit-code",
                "1",
                "--python",
                "xtask/blender/human.py",
                "--",
                "hero.json",
                "hero.blend",
            ]
            .map(OsString::from)
        );
    }

    #[test]
    fn a_missing_program_is_an_error_that_names_the_variable() {
        let program = Program(PathBuf::from("vespera-test-no-such-blender"));

        let message = format!(
            "{:#}",
            program.run(&[OsString::from("--version")]).unwrap_err()
        );

        assert!(message.contains("BLENDER"), "{message}");
    }
}
