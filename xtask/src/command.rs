use std::path::Path;

use anyhow::{Result, bail, ensure};

use crate::assets::{self, Curl};
use crate::comments::{self, Allowlist};
use crate::finding::Finding;
use crate::{docs, layers, repository, text_only};

const COMMANDS: &str = "check-text-only, check-comments, check-docs, check-layers, fetch-assets";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Check {
    TextOnly,
    Comments,
    Docs,
    Layers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task {
    Check(Check),
    FetchAssets,
}

pub fn run(arguments: impl Iterator<Item = String>) -> Result<()> {
    let root = repository::root();
    match parse(arguments)? {
        Task::Check(check) => run_check(check, &root),
        Task::FetchAssets => fetch_assets(&root),
    }
}

fn run_check(check: Check, root: &Path) -> Result<()> {
    let files = repository::files(root)?;
    let findings = match check {
        Check::TextOnly => text_only::check(&files),
        Check::Comments => comments::check(&files, &Allowlist::from_files(&files)?),
        Check::Docs => docs::check(&files),
        Check::Layers => layers::check(
            root,
            &layers::cargo_metadata(root)?,
            &layers::rules_text(&files)?,
        )?,
    };
    report(&findings)
}

fn fetch_assets(root: &Path) -> Result<()> {
    assets::fetch(root, &Curl, &mut |line| println!("{line}"))
}

fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Task> {
    let Some(name) = arguments.next() else {
        bail!("no command. Use one of these commands: {COMMANDS}");
    };
    let task = match name.as_str() {
        "check-text-only" => Task::Check(Check::TextOnly),
        "check-comments" => Task::Check(Check::Comments),
        "check-docs" => Task::Check(Check::Docs),
        "check-layers" => Task::Check(Check::Layers),
        "fetch-assets" => Task::FetchAssets,
        unknown => bail!("unknown command: {unknown}. Use one of these commands: {COMMANDS}"),
    };
    ensure!(
        arguments.next().is_none(),
        "the command {name} takes no arguments"
    );
    Ok(task)
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
    use super::{Check, Task, parse, report};
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
            Task::Check(Check::TextOnly)
        );
    }

    #[test]
    fn check_comments_is_a_command() {
        assert_eq!(
            parse(arguments(&["check-comments"])).unwrap(),
            Task::Check(Check::Comments)
        );
    }

    #[test]
    fn check_docs_is_a_command() {
        assert_eq!(
            parse(arguments(&["check-docs"])).unwrap(),
            Task::Check(Check::Docs)
        );
    }

    #[test]
    fn check_layers_is_a_command() {
        assert_eq!(
            parse(arguments(&["check-layers"])).unwrap(),
            Task::Check(Check::Layers)
        );
    }

    #[test]
    fn fetch_assets_is_a_command() {
        assert_eq!(
            parse(arguments(&["fetch-assets"])).unwrap(),
            Task::FetchAssets
        );
    }

    #[test]
    fn an_unknown_command_is_an_error_that_lists_the_commands() {
        let error = parse(arguments(&["fly"])).unwrap_err().to_string();

        assert!(error.contains("unknown command: fly"));
        assert!(error.contains("check-text-only"));
        assert!(error.contains("fetch-assets"));
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
