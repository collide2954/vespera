use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::finding::Finding;
use crate::repository::RepositoryFile;

pub const RULES_PATH: &str = "layers.toml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    #[serde(default)]
    tools: BTreeSet<String>,
    #[serde(default)]
    crates: BTreeMap<String, CrateRule>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct CrateRule {
    layer: u8,
    side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Side {
    Shared,
    Server,
    Client,
    Test,
}

impl Side {
    fn name(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::Server => "server",
            Self::Client => "client",
            Self::Test => "test",
        }
    }
}

#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
struct Dependency {
    name: String,
    kind: Option<String>,
}

pub fn cargo_metadata(root: &Path) -> Result<String> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let output = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .context("cannot run cargo metadata")?;
    ensure!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    String::from_utf8(output.stdout).context("cargo metadata gave output that is not UTF-8")
}

pub fn rules_text(files: &[RepositoryFile]) -> Result<String> {
    let file = files
        .iter()
        .find(|file| file.path == RULES_PATH)
        .with_context(|| format!("{RULES_PATH} is missing"))?;
    String::from_utf8(file.bytes.clone()).with_context(|| format!("{RULES_PATH} is not UTF-8 text"))
}

pub fn check(root: &Path, metadata: &str, rules: &str) -> Result<Vec<Finding>> {
    let metadata: Metadata =
        serde_json::from_str(metadata).context("the cargo metadata is not valid")?;
    let rules: Rules =
        toml::from_str(rules).with_context(|| format!("{RULES_PATH} is not valid"))?;
    let mut findings = entry_findings(&metadata, &rules);
    for package in &metadata.packages {
        let Some(rule) = rules.crates.get(&package.name) else {
            continue;
        };
        let path = relative(root, &package.manifest_path);
        for dependency in package
            .dependencies
            .iter()
            .filter(|dependency| dependency.kind.as_deref() != Some("dev"))
        {
            if let Some(message) = problem(&package.name, *rule, &dependency.name, &rules) {
                findings.push(Finding::in_file(&path, message));
            }
        }
    }
    Ok(findings)
}

fn entry_findings(metadata: &Metadata, rules: &Rules) -> Vec<Finding> {
    let names: BTreeSet<&str> = metadata
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    let missing = names
        .iter()
        .filter(|name| !rules.tools.contains(**name) && !rules.crates.contains_key(**name))
        .map(|name| Finding::in_file(RULES_PATH, format!("the crate {name} has no entry")));
    let stale = rules
        .tools
        .iter()
        .chain(rules.crates.keys())
        .filter(|name| !names.contains(name.as_str()))
        .map(|name| {
            Finding::in_file(
                RULES_PATH,
                format!("the entry {name} is not a workspace crate"),
            )
        });
    missing.chain(stale).collect()
}

fn problem(name: &str, rule: CrateRule, dependency: &str, rules: &Rules) -> Option<String> {
    if rules.tools.contains(dependency) {
        return Some(format!("{name} uses the tool crate {dependency}"));
    }
    let target = rules.crates.get(dependency)?;
    if target.side == Side::Test && rule.side != Side::Test {
        return Some(format!(
            "{name} uses the test crate {dependency}. Only dev-dependencies can use a test crate."
        ));
    }
    if target.layer > rule.layer {
        return Some(format!(
            "{name} (layer {}) uses {dependency} (layer {}). A crate can use only crates in the same layer or in lower layers.",
            rule.layer, target.layer
        ));
    }
    let crosses_sides = matches!(
        (rule.side, target.side),
        (Side::Server, Side::Client)
            | (Side::Client, Side::Server)
            | (Side::Shared, Side::Server | Side::Client)
    );
    crosses_sides.then(|| {
        format!(
            "the {} crate {name} uses the {} crate {dependency}",
            rule.side.name(),
            target.side.name()
        )
    })
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::{Value, json};

    use super::{check, rules_text};
    use crate::finding::Finding;
    use crate::repository::RepositoryFile;

    const NO_FINDINGS: [&str; 0] = [];

    type Dependencies<'a> = &'a [(&'a str, Option<&'a str>)];

    fn package(name: &str, dependencies: Dependencies) -> Value {
        json!({
            "name": name,
            "manifest_path": format!("/repo/crates/{name}/Cargo.toml"),
            "dependencies": dependencies
                .iter()
                .map(|(dependency, kind)| json!({ "name": dependency, "kind": kind }))
                .collect::<Vec<_>>(),
        })
    }

    fn findings(packages: &[Value], rules: &str) -> Vec<String> {
        let metadata = json!({ "packages": packages }).to_string();
        check(Path::new("/repo"), &metadata, rules)
            .unwrap()
            .iter()
            .map(Finding::to_string)
            .collect()
    }

    const RULES: &str = r#"
tools = ["xtask"]

[crates]
vespera_core = { layer = 1, side = "shared" }
vespera_rules = { layer = 2, side = "shared" }
vespera_content = { layer = 2, side = "shared" }
vespera_server = { layer = 4, side = "server" }
vespera_client = { layer = 4, side = "client" }
vespera_testkit = { layer = 4, side = "test" }
"#;

    fn workspace(changed: &[(&str, Dependencies)]) -> Vec<Value> {
        let names = [
            "vespera_core",
            "vespera_rules",
            "vespera_content",
            "vespera_server",
            "vespera_client",
            "vespera_testkit",
            "xtask",
        ];
        names
            .iter()
            .map(|name| {
                let dependencies = changed
                    .iter()
                    .find(|(changed_name, _)| changed_name == name)
                    .map_or(&[][..], |(_, dependencies)| *dependencies);
                package(name, dependencies)
            })
            .collect()
    }

    #[test]
    fn a_crate_can_use_a_crate_in_a_lower_layer() {
        let packages = workspace(&[("vespera_rules", &[("vespera_core", None)])]);

        assert_eq!(findings(&packages, RULES), NO_FINDINGS);
    }

    #[test]
    fn a_crate_can_use_a_crate_in_the_same_layer() {
        let packages = workspace(&[("vespera_rules", &[("vespera_content", None)])]);

        assert_eq!(findings(&packages, RULES), NO_FINDINGS);
    }

    #[test]
    fn a_crate_cannot_use_a_crate_in_a_higher_layer() {
        let packages = workspace(&[("vespera_core", &[("vespera_rules", None)])]);

        assert_eq!(
            findings(&packages, RULES),
            [
                "crates/vespera_core/Cargo.toml: vespera_core (layer 1) uses vespera_rules (layer 2). A crate can use only crates in the same layer or in lower layers."
            ]
        );
    }

    #[test]
    fn a_server_crate_cannot_use_a_client_crate() {
        let packages = workspace(&[("vespera_server", &[("vespera_client", None)])]);

        assert_eq!(
            findings(&packages, RULES),
            [
                "crates/vespera_server/Cargo.toml: the server crate vespera_server uses the client crate vespera_client"
            ]
        );
    }

    #[test]
    fn a_client_crate_cannot_use_a_server_crate() {
        let packages = workspace(&[("vespera_client", &[("vespera_server", None)])]);

        assert_eq!(
            findings(&packages, RULES),
            [
                "crates/vespera_client/Cargo.toml: the client crate vespera_client uses the server crate vespera_server"
            ]
        );
    }

    #[test]
    fn a_shared_crate_cannot_use_a_server_crate() {
        let rules = "[crates]\nvespera_rules = { layer = 4, side = \"shared\" }\nvespera_server = { layer = 4, side = \"server\" }\n";
        let packages = [
            package("vespera_rules", &[("vespera_server", None)]),
            package("vespera_server", &[]),
        ];

        assert_eq!(
            findings(&packages, rules),
            [
                "crates/vespera_rules/Cargo.toml: the shared crate vespera_rules uses the server crate vespera_server"
            ]
        );
    }

    #[test]
    fn a_test_crate_can_use_server_and_client_crates() {
        let packages = workspace(&[(
            "vespera_testkit",
            &[("vespera_server", None), ("vespera_client", None)],
        )]);

        assert_eq!(findings(&packages, RULES), NO_FINDINGS);
    }

    #[test]
    fn a_normal_dependency_cannot_use_a_test_crate() {
        let packages = workspace(&[("vespera_server", &[("vespera_testkit", None)])]);

        assert_eq!(
            findings(&packages, RULES),
            [
                "crates/vespera_server/Cargo.toml: vespera_server uses the test crate vespera_testkit. Only dev-dependencies can use a test crate."
            ]
        );
    }

    #[test]
    fn dev_dependencies_are_not_checked() {
        let packages = workspace(&[(
            "vespera_core",
            &[
                ("vespera_testkit", Some("dev")),
                ("vespera_server", Some("dev")),
            ],
        )]);

        assert_eq!(findings(&packages, RULES), NO_FINDINGS);
    }

    #[test]
    fn build_dependencies_are_checked() {
        let packages = workspace(&[("vespera_core", &[("vespera_rules", Some("build"))])]);

        assert_eq!(findings(&packages, RULES).len(), 1);
    }

    #[test]
    fn no_crate_can_use_a_tool() {
        let packages = workspace(&[("vespera_server", &[("xtask", None)])]);

        assert_eq!(
            findings(&packages, RULES),
            ["crates/vespera_server/Cargo.toml: vespera_server uses the tool crate xtask"]
        );
    }

    #[test]
    fn external_crates_are_not_checked() {
        let packages = workspace(&[("vespera_core", &[("bevy", None)])]);

        assert_eq!(findings(&packages, RULES), NO_FINDINGS);
    }

    #[test]
    fn each_workspace_crate_has_an_entry() {
        let mut packages = workspace(&[]);
        packages.push(package("vespera_world", &[]));

        assert_eq!(
            findings(&packages, RULES),
            ["layers.toml: the crate vespera_world has no entry"]
        );
    }

    #[test]
    fn each_entry_names_a_workspace_crate() {
        let packages: Vec<Value> = workspace(&[])
            .into_iter()
            .filter(|package| package["name"] != "vespera_content")
            .collect();

        assert_eq!(
            findings(&packages, RULES),
            ["layers.toml: the entry vespera_content is not a workspace crate"]
        );
    }

    #[test]
    fn the_rules_come_from_the_rules_file() {
        let files = [RepositoryFile {
            path: "layers.toml".to_owned(),
            bytes: b"tools = [\"xtask\"]\n".to_vec(),
        }];

        assert_eq!(rules_text(&files).unwrap(), "tools = [\"xtask\"]\n");
    }

    #[test]
    fn a_missing_rules_file_is_an_error() {
        assert!(rules_text(&[]).is_err());
    }

    #[test]
    fn rules_with_an_unknown_side_are_an_error() {
        let metadata = json!({ "packages": [] }).to_string();
        let rules = "[crates]\nvespera_core = { layer = 1, side = \"middle\" }\n";

        assert!(check(Path::new("/repo"), &metadata, rules).is_err());
    }
}
