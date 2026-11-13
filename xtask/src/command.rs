use anyhow::{Result, bail, ensure};

use crate::finding::Finding;
use crate::{repository, text_only};

const COMMANDS: &str = "check-text-only";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    CheckTextOnly,
}

pub fn run(arguments: impl Iterator<Item = String>) -> Result<()> {
    let command = parse(arguments)?;
    let files = repository::files(&repository::root())?;
    let findings = match command {
        Command::CheckTextOnly => text_only::check(&files),
    };
    report(&findings)
}

fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Command> {
    let Some(name) = arguments.next() else {
        bail!("no command. Use one of these commands: {COMMANDS}");
    };
    let command = match name.as_str() {
        "check-text-only" => Command::CheckTextOnly,
        unknown => bail!("unknown command: {unknown}. Use one of these commands: {COMMANDS}"),
    };
    ensure!(
        arguments.next().is_none(),
        "the command {name} takes no arguments"
    );
    Ok(command)
}

fn report(findings: &[Finding]) -> Result<()> {
    for finding in findings {
        println!("{finding}");
    }
    let noun = if findings.len() == 1 {
        "problem"
    } else {
        "problems"
    };
    ensure!(findings.is_empty(), "{} {noun}", findings.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Command, parse, report};
    use crate::finding::Finding;

    fn arguments(values: &[&str]) -> impl Iterator<Item = String> {
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn check_text_only_is_a_command() {
        assert_eq!(
            parse(arguments(&["check-text-only"])).unwrap(),
            Command::CheckTextOnly
        );
    }

    #[test]
    fn an_unknown_command_is_an_error_that_lists_the_commands() {
        let error = parse(arguments(&["fly"])).unwrap_err().to_string();

        assert!(error.contains("unknown command: fly"));
        assert!(error.contains("check-text-only"));
    }

    #[test]
    fn no_command_is_an_error() {
        assert!(parse(arguments(&[])).is_err());
    }

    #[test]
    fn an_extra_argument_is_an_error() {
        assert!(parse(arguments(&["check-text-only", "extra"])).is_err());
    }

    #[test]
    fn a_report_without_findings_passes() {
        assert!(report(&[]).is_ok());
    }

    #[test]
    fn a_report_with_one_finding_fails_with_one_problem() {
        let findings = [Finding::in_file("a.png", "the file is not UTF-8 text")];

        assert_eq!(report(&findings).unwrap_err().to_string(), "1 problem");
    }

    #[test]
    fn a_report_with_findings_fails_and_counts_them() {
        let findings = [
            Finding::in_file("a.png", "the file is not UTF-8 text"),
            Finding::in_file("b.png", "the file is not UTF-8 text"),
        ];

        assert_eq!(report(&findings).unwrap_err().to_string(), "2 problems");
    }
}
