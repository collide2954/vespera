use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};

use crate::assets::{self, Curl};
use crate::comments::{self, Allowlist};
use crate::finding::Finding;
use crate::import::{self, Program};
use crate::{bench, docs, dxc, layers, repository, text_only};

const COMMANDS: &str = "bench, check-text-only, check-comments, check-docs, check-layers, fetch-assets, fetch-dxc, import-assets";
const BENCH_SECONDS: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Check {
    TextOnly,
    Comments,
    Docs,
    Layers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Task {
    Bench { seconds: u64 },
    Check(Check),
    FetchAssets,
    FetchDxc,
    ImportAssets { sources: PathBuf },
}

pub fn run(arguments: impl Iterator<Item = String>) -> Result<()> {
    let root = repository::root();
    match parse(arguments)? {
        Task::Bench { seconds } => bench::run(&root, seconds),
        Task::Check(check) => run_check(check, &root),
        Task::FetchAssets => assets::fetch(&root, &Curl, &mut print_line),
        Task::FetchDxc => dxc::fetch(&root, &Curl, &mut print_line),
        Task::ImportAssets { sources } => import::run(
            &root,
            &sources,
            &Program::from_environment(),
            &mut print_line,
        ),
    }
}

fn print_line(line: String) {
    println!("{line}");
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

fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Task> {
    let Some(name) = arguments.next() else {
        bail!("no command. Use one of these commands: {COMMANDS}");
    };
    let task = match name.as_str() {
        "bench" => Task::Bench {
            seconds: bench_seconds(arguments.next())?,
        },
        "check-text-only" => Task::Check(Check::TextOnly),
        "check-comments" => Task::Check(Check::Comments),
        "check-docs" => Task::Check(Check::Docs),
        "check-layers" => Task::Check(Check::Layers),
        "fetch-assets" => Task::FetchAssets,
        "fetch-dxc" => Task::FetchDxc,
        "import-assets" => Task::ImportAssets {
            sources: arguments.next().map(PathBuf::from).context(
                "import-assets needs the folder of the source packs, with 100style/100STYLE.zip in it",
            )?,
        },
        unknown => bail!("unknown command: {unknown}. Use one of these commands: {COMMANDS}"),
    };
    ensure!(
        arguments.next().is_none(),
        "too many arguments for the command {name}"
    );
    Ok(task)
}

fn bench_seconds(argument: Option<String>) -> Result<u64> {
    let Some(text) = argument else {
        return Ok(BENCH_SECONDS);
    };
    let seconds: u64 = text
        .parse()
        .with_context(|| format!("the benchmark time {text} is not a whole number of seconds"))?;
    ensure!(seconds > 0, "the benchmark time must be more than 0 s");
    Ok(seconds)
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
    use std::path::PathBuf;

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
    fn fetch_dxc_is_a_command() {
        assert_eq!(parse(arguments(&["fetch-dxc"])).unwrap(), Task::FetchDxc);
    }

    #[test]
    fn import_assets_takes_the_folder_of_the_source_packs() {
        assert_eq!(
            parse(arguments(&["import-assets", "C:/vespera-sources"])).unwrap(),
            Task::ImportAssets {
                sources: PathBuf::from("C:/vespera-sources")
            }
        );
    }

    #[test]
    fn import_assets_without_a_folder_is_an_error_that_names_the_archive() {
        let error = format!("{:#}", parse(arguments(&["import-assets"])).unwrap_err());

        assert!(error.contains("100STYLE.zip"), "{error}");
    }

    #[test]
    fn import_assets_rejects_a_second_folder() {
        assert!(parse(arguments(&["import-assets", "a", "b"])).is_err());
    }

    #[test]
    fn bench_is_a_command_with_30_seconds_by_default() {
        assert_eq!(
            parse(arguments(&["bench"])).unwrap(),
            Task::Bench { seconds: 30 }
        );
    }

    #[test]
    fn bench_takes_a_time_in_seconds() {
        assert_eq!(
            parse(arguments(&["bench", "60"])).unwrap(),
            Task::Bench { seconds: 60 }
        );
    }

    #[test]
    fn bench_rejects_a_time_that_is_not_a_positive_whole_number() {
        for bad in ["0", "-5", "ten", "1.5"] {
            assert!(parse(arguments(&["bench", bad])).is_err(), "{bad}");
        }
    }

    #[test]
    fn bench_rejects_a_second_argument() {
        assert!(parse(arguments(&["bench", "30", "extra"])).is_err());
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
