use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

pub fn root() -> PathBuf {
    let manifest_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_directory
        .parent()
        .unwrap_or(manifest_directory)
        .to_path_buf()
}

pub fn files(root: &Path) -> Result<Vec<RepositoryFile>> {
    let mut files = Vec::new();
    for path in listed_paths(root)? {
        let full_path = root.join(&path);
        if full_path.is_file() {
            let bytes = fs::read(&full_path).with_context(|| format!("cannot read {path}"))?;
            files.push(RepositoryFile { path, bytes });
        }
    }
    Ok(files)
}

fn listed_paths(root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(root)
        .output()
        .with_context(|| format!("cannot run git in {}", root.display()))?;
    ensure!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let listing =
        String::from_utf8(output.stdout).context("git ls-files gave a path that is not UTF-8")?;
    let mut paths: Vec<String> = listing
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use super::{files, root};

    fn git(directory: &Path, arguments: &[&str]) {
        let status = Command::new("git")
            .args(arguments)
            .current_dir(directory)
            .status()
            .unwrap();
        assert!(status.success());
    }

    fn repository_with(entries: &[(&str, &str)]) -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        git(directory.path(), &["init", "--quiet"]);
        for (path, contents) in entries {
            fs::write(directory.path().join(path), contents).unwrap();
        }
        directory
    }

    #[test]
    fn the_root_contains_the_workspace_manifest() {
        let manifest = fs::read_to_string(root().join("Cargo.toml")).unwrap();

        assert!(manifest.contains("[workspace]"));
    }

    #[test]
    fn files_include_new_files_but_not_ignored_files() {
        let repository = repository_with(&[
            (".gitignore", "ignored.txt\n"),
            ("kept.txt", "kept"),
            ("ignored.txt", "ignored"),
        ]);

        let paths: Vec<String> = files(repository.path())
            .unwrap()
            .into_iter()
            .map(|file| file.path)
            .collect();

        assert_eq!(paths, [".gitignore", "kept.txt"]);
    }

    #[test]
    fn files_contain_the_bytes_of_each_file() {
        let repository = repository_with(&[("notes.md", "starlight")]);

        let listed = files(repository.path()).unwrap();

        assert_eq!(listed[0].bytes, b"starlight");
    }

    #[test]
    fn files_skip_tracked_files_that_were_deleted() {
        let repository = repository_with(&[("gone.txt", "gone"), ("kept.txt", "kept")]);
        git(repository.path(), &["add", "gone.txt"]);
        fs::remove_file(repository.path().join("gone.txt")).unwrap();

        let paths: Vec<String> = files(repository.path())
            .unwrap()
            .into_iter()
            .map(|file| file.path)
            .collect();

        assert_eq!(paths, ["kept.txt"]);
    }

    #[test]
    fn files_fail_for_a_missing_directory() {
        let directory = tempfile::tempdir().unwrap();

        assert!(files(&directory.path().join("missing")).is_err());
    }
}
