mod fetch;
mod manifest;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use fetch::{Outcome, fetch_archive};
use manifest::Manifest;

pub use fetch::{Curl, Download};

pub const MANIFEST: &str = "assets.toml";
pub const CACHE: &str = "assets";

pub fn fetch(root: &Path, source: &impl Download, report: &mut impl FnMut(String)) -> Result<()> {
    let text = fs::read_to_string(root.join(MANIFEST))
        .with_context(|| format!("cannot read {MANIFEST}"))?;
    let manifest = Manifest::parse(&text)?;
    if manifest.archives.is_empty() {
        report(format!("{MANIFEST} lists no archives"));
    }
    let cache = root.join(CACHE);
    for archive in &manifest.archives {
        let label = format!("{} version {}", archive.name, archive.version);
        if fetch::is_current(&cache.join(&archive.name), &archive.sha256) {
            report(format!("{label} is current"));
            continue;
        }
        report(format!(
            "downloading {label} ({} MB)",
            archive.size.div_ceil(1_000_000)
        ));
        if fetch_archive(archive, &cache, source)? == Outcome::Fetched {
            report(format!("fetched {label}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use anyhow::Result;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::fetch::sha256_of;
    use super::manifest::Manifest;
    use super::{CACHE, Download, MANIFEST, fetch};
    use crate::repository;

    struct LocalFile(PathBuf);

    impl Download for LocalFile {
        fn download(&self, _url: &str, destination: &Path) -> Result<()> {
            fs::copy(&self.0, destination)?;
            Ok(())
        }
    }

    fn root_with_one_archive() -> (tempfile::TempDir, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let zip = root.path().join("valley-1.zip");
        let mut writer = ZipWriter::new(File::create(&zip).unwrap());
        writer
            .start_file("rocks/boulder.txt", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"granite").unwrap();
        writer.finish().unwrap();
        let manifest = format!(
            "[[archive]]\nname = \"valley\"\nversion = 1\nurl = \"https://assets.test/valley-1.zip\"\nsha256 = \"{}\"\nsize = {}\nlicenses = [\"CC0-1.0\"]\n",
            sha256_of(&zip).unwrap(),
            fs::metadata(&zip).unwrap().len()
        );
        fs::write(root.path().join(MANIFEST), manifest).unwrap();
        (root, zip)
    }

    fn run(root: &Path, source: &impl Download) -> Result<Vec<String>> {
        let mut lines = Vec::new();
        fetch(root, source, &mut |line| lines.push(line))?;
        Ok(lines)
    }

    #[test]
    fn fetch_extracts_each_archive_that_the_manifest_lists() {
        let (root, zip) = root_with_one_archive();

        let lines = run(root.path(), &LocalFile(zip)).unwrap();

        assert_eq!(
            lines,
            [
                "downloading valley version 1 (1 MB)",
                "fetched valley version 1"
            ]
        );
        assert_eq!(
            fs::read_to_string(root.path().join(CACHE).join("valley/rocks/boulder.txt")).unwrap(),
            "granite"
        );
    }

    #[test]
    fn a_second_fetch_reports_that_the_archive_is_current() {
        let (root, zip) = root_with_one_archive();
        run(root.path(), &LocalFile(zip.clone())).unwrap();

        let lines = run(root.path(), &LocalFile(zip)).unwrap();

        assert_eq!(lines, ["valley version 1 is current"]);
    }

    #[test]
    fn a_manifest_without_archives_says_so() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join(MANIFEST), "").unwrap();

        let lines = run(root.path(), &LocalFile(PathBuf::new())).unwrap();

        assert_eq!(lines, ["assets.toml lists no archives"]);
    }

    #[test]
    fn a_missing_manifest_is_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = run(root.path(), &LocalFile(PathBuf::new())).unwrap_err();

        assert!(format!("{error:#}").contains(MANIFEST));
    }

    #[test]
    fn the_repository_manifest_is_valid() {
        let text = fs::read_to_string(repository::root().join(MANIFEST)).unwrap();

        assert!(Manifest::parse(&text).is_ok());
    }
}
