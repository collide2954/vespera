use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use super::manifest::Archive;

pub const STAMP: &str = ".sha256";

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

pub trait Download {
    fn download(&self, url: &str, destination: &Path) -> Result<()>;
}

pub struct Curl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Fetched,
    Current,
}

struct Work {
    download: PathBuf,
    staging: PathBuf,
}

impl Download for Curl {
    fn download(&self, url: &str, destination: &Path) -> Result<()> {
        let output = Command::new("curl")
            .args([
                "--fail",
                "--location",
                "--silent",
                "--show-error",
                "--output",
            ])
            .arg(destination)
            .arg(url)
            .output()
            .context("cannot start curl. Install curl and try again")?;
        ensure!(
            output.status.success(),
            "curl cannot download {url}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Ok(())
    }
}

impl Work {
    fn clean(&self) -> Result<()> {
        remove_file_if_present(&self.download)?;
        remove_folder_if_present(&self.staging)
    }
}

pub fn fetch_archive(archive: &Archive, cache: &Path, source: &impl Download) -> Result<Outcome> {
    let folder = cache.join(&archive.name);
    if is_current(&folder, &archive.sha256) {
        return Ok(Outcome::Current);
    }
    fs::create_dir_all(cache).with_context(|| format!("cannot create {}", cache.display()))?;
    let work = Work {
        download: cache.join(format!(".{}.zip", archive.name)),
        staging: cache.join(format!(".{}", archive.name)),
    };
    work.clean()?;
    let installed = install(archive, source, &work, &folder);
    work.clean()?;
    installed.map(|()| Outcome::Fetched)
}

pub fn sha256_of(path: &Path) -> Result<String> {
    let mut file = BufReader::new(
        File::open(path).with_context(|| format!("cannot open {}", path.display()))?,
    );
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 16];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("cannot read {}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .flat_map(|byte| {
            [
                HEX_DIGITS[usize::from(byte >> 4)],
                HEX_DIGITS[usize::from(byte & 0x0f)],
            ]
        })
        .map(char::from)
        .collect())
}

pub fn is_current(folder: &Path, sha256: &str) -> bool {
    fs::read_to_string(folder.join(STAMP)).is_ok_and(|stamp| stamp == sha256)
}

fn install(archive: &Archive, source: &impl Download, work: &Work, folder: &Path) -> Result<()> {
    source
        .download(&archive.url, &work.download)
        .with_context(|| format!("cannot download the archive {}", archive.name))?;
    verify(archive, &work.download)?;
    extract(&work.download, &work.staging)
        .with_context(|| format!("cannot extract the archive {}", archive.name))?;
    fs::write(work.staging.join(STAMP), &archive.sha256)
        .with_context(|| format!("cannot write the stamp of {}", archive.name))?;
    remove_folder_if_present(folder)?;
    fs::rename(&work.staging, folder)
        .with_context(|| format!("cannot move the archive {} into place", archive.name))
}

fn verify(archive: &Archive, file: &Path) -> Result<()> {
    let size = fs::metadata(file)
        .with_context(|| format!("cannot read the size of {}", file.display()))?
        .len();
    ensure!(
        size == archive.size,
        "the archive {} has {size} bytes, but assets.toml gives {} bytes",
        archive.name,
        archive.size
    );
    let sha256 = sha256_of(file)?;
    ensure!(
        sha256 == archive.sha256,
        "the archive {} has the SHA-256 hash {sha256}, but assets.toml gives {}",
        archive.name,
        archive.sha256
    );
    Ok(())
}

fn extract(archive: &Path, destination: &Path) -> Result<()> {
    let file = File::open(archive).with_context(|| format!("cannot open {}", archive.display()))?;
    let mut zip = ZipArchive::new(BufReader::new(file))?;
    fs::create_dir_all(destination)?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;
        let Some(relative) = entry.enclosed_name() else {
            bail!(
                "the entry {} leaves the folder of the archive",
                String::from_utf8_lossy(entry.name_raw())
            );
        };
        let path = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&path)?;
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        io::copy(&mut entry, &mut File::create(&path)?)?;
    }
    Ok(())
}

fn remove_file_if_present(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            Err(error).with_context(|| format!("cannot remove {}", path.display()))
        }
        _ => Ok(()),
    }
}

fn remove_folder_if_present(path: &Path) -> Result<()> {
    match fs::remove_dir_all(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            Err(error).with_context(|| format!("cannot remove {}", path.display()))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use anyhow::{Result, bail};
    use tempfile::TempDir;

    use super::{Curl, Download, Outcome, STAMP, fetch_archive, sha256_of};
    use crate::assets::manifest::Archive;
    use crate::assets::testing::{LocalFile, Offline, zip_with};

    struct Broken;

    impl Download for Broken {
        fn download(&self, _url: &str, destination: &Path) -> Result<()> {
            fs::write(destination, "partial")?;
            bail!("the connection closed")
        }
    }

    fn archive_of(path: &Path) -> Archive {
        Archive {
            name: "valley".to_owned(),
            version: 1,
            url: "https://assets.test/valley-1.zip".to_owned(),
            sha256: sha256_of(path).unwrap(),
            size: fs::metadata(path).unwrap().len(),
            licenses: vec!["CC0-1.0".to_owned()],
        }
    }

    fn setup(entries: &[(&str, &str)]) -> (TempDir, PathBuf, Archive) {
        let directory = tempfile::tempdir().unwrap();
        let zip = zip_with(directory.path(), "valley-1.zip", entries);
        let archive = archive_of(&zip);
        (directory, zip, archive)
    }

    fn file_url(path: &Path) -> String {
        let text = path.to_str().unwrap().replace('\\', "/");
        if text.starts_with('/') {
            format!("file://{text}")
        } else {
            format!("file:///{text}")
        }
    }

    #[test]
    fn the_hash_of_abc_is_the_standard_value() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("abc.txt");
        fs::write(&path, "abc").unwrap();

        assert_eq!(
            sha256_of(&path).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn an_archive_is_extracted_into_a_folder_with_its_name() {
        let (directory, zip, archive) = setup(&[("rocks/boulder.txt", "granite")]);
        let cache = directory.path().join("assets");

        let outcome = fetch_archive(&archive, &cache, &LocalFile(zip)).unwrap();

        assert_eq!(outcome, Outcome::Fetched);
        assert_eq!(
            fs::read_to_string(cache.join("valley/rocks/boulder.txt")).unwrap(),
            "granite"
        );
    }

    #[test]
    fn the_folder_of_an_archive_has_a_stamp_with_the_hash() {
        let (directory, zip, archive) = setup(&[("a.txt", "a")]);
        let cache = directory.path().join("assets");

        fetch_archive(&archive, &cache, &LocalFile(zip)).unwrap();

        assert_eq!(
            fs::read_to_string(cache.join("valley").join(STAMP)).unwrap(),
            archive.sha256
        );
    }

    #[test]
    fn a_wrong_hash_is_an_error_that_names_both_hashes_and_leaves_no_files() {
        let (directory, zip, mut archive) = setup(&[("a.txt", "a")]);
        let actual = archive.sha256.clone();
        archive.sha256 = "0".repeat(64);
        let cache = directory.path().join("assets");

        let error = fetch_archive(&archive, &cache, &LocalFile(zip)).unwrap_err();
        let message = format!("{error:#}");

        assert!(message.contains(&actual));
        assert!(message.contains(&"0".repeat(64)));
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
    }

    #[test]
    fn a_wrong_size_is_an_error() {
        let (directory, zip, mut archive) = setup(&[("a.txt", "a")]);
        archive.size += 1;
        let cache = directory.path().join("assets");

        let error = fetch_archive(&archive, &cache, &LocalFile(zip)).unwrap_err();

        assert!(format!("{error:#}").contains("bytes"));
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
    }

    #[test]
    fn a_current_archive_is_not_downloaded_again() {
        let (directory, zip, archive) = setup(&[("a.txt", "a")]);
        let cache = directory.path().join("assets");
        fetch_archive(&archive, &cache, &LocalFile(zip)).unwrap();

        let outcome = fetch_archive(&archive, &cache, &Offline).unwrap();

        assert_eq!(outcome, Outcome::Current);
    }

    #[test]
    fn a_changed_archive_replaces_the_old_files() {
        let (directory, old, archive) = setup(&[("old.txt", "old")]);
        let cache = directory.path().join("assets");
        fetch_archive(&archive, &cache, &LocalFile(old)).unwrap();
        let new = zip_with(directory.path(), "valley-2.zip", &[("new.txt", "new")]);
        let changed = Archive {
            version: 2,
            ..archive_of(&new)
        };

        fetch_archive(&changed, &cache, &LocalFile(new)).unwrap();

        assert!(!cache.join("valley/old.txt").exists());
        assert_eq!(
            fs::read_to_string(cache.join("valley/new.txt")).unwrap(),
            "new"
        );
    }

    #[test]
    fn an_entry_that_leaves_the_folder_is_an_error() {
        let (directory, zip, archive) = setup(&[("../escape.txt", "out")]);
        let cache = directory.path().join("assets");

        assert!(fetch_archive(&archive, &cache, &LocalFile(zip)).is_err());
        assert!(!directory.path().join("escape.txt").exists());
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
    }

    #[test]
    fn a_failed_download_is_an_error_and_leaves_no_files() {
        let (directory, _zip, archive) = setup(&[("a.txt", "a")]);
        let cache = directory.path().join("assets");

        let error = fetch_archive(&archive, &cache, &Broken).unwrap_err();

        assert!(format!("{error:#}").contains("the connection closed"));
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
    }

    #[test]
    fn curl_downloads_a_file_url() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.bin");
        fs::write(&source, "vesper").unwrap();
        let destination = directory.path().join("copy.bin");

        Curl.download(&file_url(&source), &destination).unwrap();

        assert_eq!(fs::read_to_string(&destination).unwrap(), "vesper");
    }

    #[test]
    fn curl_fails_with_a_message_for_a_missing_file() {
        let directory = tempfile::tempdir().unwrap();
        let missing = directory.path().join("missing.bin");
        let destination = directory.path().join("copy.bin");

        let error = Curl
            .download(&file_url(&missing), &destination)
            .unwrap_err();

        assert!(format!("{error:#}").contains("curl"));
    }
}
