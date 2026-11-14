use std::collections::HashSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

pub const APPROVED_LICENSES: [&str; 3] = ["CC0-1.0", "CC-BY-4.0", "OFL-1.1"];

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(default, rename = "archive")]
    pub archives: Vec<Archive>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    pub name: String,
    pub version: u32,
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub licenses: Vec<String>,
}

impl Manifest {
    pub fn parse(text: &str) -> Result<Self> {
        let manifest: Self = toml::from_str(text).context("assets.toml has a bad format")?;
        let mut names = HashSet::new();
        for archive in &manifest.archives {
            archive.validate()?;
            ensure!(
                names.insert(archive.name.as_str()),
                "assets.toml lists the archive {} two times",
                archive.name
            );
        }
        Ok(manifest)
    }
}

impl Archive {
    fn validate(&self) -> Result<()> {
        let name = &self.name;
        ensure!(
            is_snake_case(name),
            "the archive name {name:?} is not lowercase snake case"
        );
        ensure!(
            self.version > 0,
            "the archive {name} has version 0. Versions start at 1"
        );
        ensure!(
            self.url.starts_with("https://"),
            "the archive {name} has the URL {}. Use an https URL",
            self.url
        );
        ensure!(
            is_sha256(&self.sha256),
            "the archive {name} has a bad SHA-256 hash. Use 64 lowercase hex digits"
        );
        ensure!(self.size > 0, "the archive {name} has the size 0");
        ensure!(
            !self.licenses.is_empty(),
            "the archive {name} has no license"
        );
        for license in &self.licenses {
            ensure!(
                APPROVED_LICENSES.contains(&license.as_str()),
                "the archive {name} has the license {license}, which is not approved. The approved licenses are {}",
                APPROVED_LICENSES.join(", ")
            );
        }
        Ok(())
    }
}

fn is_snake_case(name: &str) -> bool {
    name.split('_').all(|word| {
        !word.is_empty()
            && word
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

fn is_sha256(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use super::Manifest;

    const HASH: &str = "b542127a8e25547c7c29c19f2d1d2adb9a664c80396ecd694095dbc8028a0107";

    fn archive(name: &str, url: &str, sha256: &str, licenses: &str) -> String {
        format!(
            "[[archive]]\nname = \"{name}\"\nversion = 1\nurl = \"{url}\"\nsha256 = \"{sha256}\"\nsize = 280737770\nlicenses = [{licenses}]\n"
        )
    }

    fn valid(name: &str) -> String {
        archive(
            name,
            "https://github.com/collide2954/vespera-assets/releases/download/a/a.zip",
            HASH,
            "\"CC0-1.0\"",
        )
    }

    fn error(text: &str) -> String {
        format!("{:#}", Manifest::parse(text).unwrap_err())
    }

    #[test]
    fn a_manifest_with_one_archive_parses() {
        let manifest = Manifest::parse(&valid("humans_source")).unwrap();

        assert_eq!(manifest.archives.len(), 1);
        assert_eq!(manifest.archives[0].name, "humans_source");
        assert_eq!(manifest.archives[0].size, 280_737_770);
        assert_eq!(manifest.archives[0].licenses, ["CC0-1.0"]);
    }

    #[test]
    fn an_empty_manifest_has_no_archives() {
        assert_eq!(
            Manifest::parse("").unwrap(),
            Manifest {
                archives: Vec::new()
            }
        );
    }

    #[test]
    fn an_archive_can_have_more_than_one_license() {
        let text = archive(
            "valley",
            "https://a.test/v.zip",
            HASH,
            "\"CC0-1.0\", \"CC-BY-4.0\"",
        );

        assert_eq!(
            Manifest::parse(&text).unwrap().archives[0].licenses,
            ["CC0-1.0", "CC-BY-4.0"]
        );
    }

    #[test]
    fn a_license_that_is_not_approved_is_an_error() {
        let text = archive("valley", "https://a.test/v.zip", HASH, "\"CC-BY-NC-4.0\"");

        let message = error(&text);

        assert!(message.contains("CC-BY-NC-4.0"));
        assert!(message.contains("CC0-1.0, CC-BY-4.0, OFL-1.1"));
    }

    #[test]
    fn an_archive_without_a_license_is_an_error() {
        assert!(error(&archive("valley", "https://a.test/v.zip", HASH, "")).contains("no license"));
    }

    #[test]
    fn a_hash_needs_64_lowercase_hex_digits() {
        let short = archive("valley", "https://a.test/v.zip", "abc", "\"CC0-1.0\"");
        let upper = archive(
            "valley",
            "https://a.test/v.zip",
            &HASH.to_uppercase(),
            "\"CC0-1.0\"",
        );

        assert!(error(&short).contains("SHA-256"));
        assert!(error(&upper).contains("SHA-256"));
    }

    #[test]
    fn a_url_needs_https() {
        let text = archive("valley", "http://a.test/v.zip", HASH, "\"CC0-1.0\"");

        assert!(error(&text).contains("http://a.test/v.zip"));
    }

    #[test]
    fn an_archive_name_needs_lowercase_snake_case() {
        for name in ["Valley", "valley-source", "valley__source", "_valley", ""] {
            assert!(
                Manifest::parse(&valid(name)).is_err(),
                "the name {name:?} must be an error"
            );
        }
    }

    #[test]
    fn two_archives_cannot_have_the_same_name() {
        let text = format!("{}\n{}", valid("valley"), valid("valley"));

        assert!(error(&text).contains("two times"));
    }

    #[test]
    fn version_zero_is_an_error() {
        let text = valid("valley").replace("version = 1", "version = 0");

        assert!(error(&text).contains("version 0"));
    }

    #[test]
    fn size_zero_is_an_error() {
        let text = valid("valley").replace("size = 280737770", "size = 0");

        assert!(error(&text).contains("size 0"));
    }

    #[test]
    fn an_unknown_field_is_an_error() {
        let text = format!("{}author = \"someone\"\n", valid("valley"));

        assert!(Manifest::parse(&text).is_err());
    }
}
