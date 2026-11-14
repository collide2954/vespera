use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::assets::{self, Archive, Download};

const FILES: [(&str, &str); 3] = [
    ("bin/x64/dxcompiler.dll", "dxcompiler.dll"),
    ("LICENSE-LLVM.txt", "dxcompiler-LICENSE-LLVM.txt"),
    ("LICENCE-MIT.txt", "dxcompiler-LICENSE-MIT.txt"),
];

pub fn release() -> Archive {
    Archive {
        name: "dxc_1_9_2609".to_owned(),
        version: 1,
        url: "https://github.com/microsoft/DirectXShaderCompiler/releases/download/v1.9.2609/dxc_2026_09_29.zip".to_owned(),
        sha256: "ad31b1fc8443175d204f77a611fdb3ef2ec42759bdc2f1167368de24a4a7e7f1".to_owned(),
        size: 32_287_592,
        licenses: vec!["MIT".to_owned(), "NCSA".to_owned()],
    }
}

pub fn fetch(root: &Path, source: &impl Download, report: &mut impl FnMut(String)) -> Result<()> {
    install(&release(), &root.join("target"), source, report)
}

fn install(
    release: &Archive,
    target: &Path,
    source: &impl Download,
    report: &mut impl FnMut(String),
) -> Result<()> {
    assets::fetch_one(release, target, source, report)?;
    let extracted = target.join(&release.name);
    for destination in destinations(target) {
        fs::create_dir_all(&destination)
            .with_context(|| format!("cannot create {}", destination.display()))?;
        for (from, to) in FILES {
            fs::copy(extracted.join(from), destination.join(to))
                .with_context(|| format!("cannot copy {from} from the DXC archive"))?;
        }
        report(format!(
            "copied dxcompiler.dll to {}",
            destination.display()
        ));
    }
    Ok(())
}

fn destinations(target: &Path) -> Vec<PathBuf> {
    let mut destinations = vec![target.join("debug")];
    let release = target.join("release");
    if release.is_dir() {
        destinations.push(release);
    }
    destinations
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use anyhow::Result;

    use super::{install, release};
    use crate::assets::testing::{LocalFile, Offline, zip_with};
    use crate::assets::{Archive, Download, sha256_of};

    const ENTRIES: [(&str, &str); 3] = [
        ("bin/x64/dxcompiler.dll", "compiler"),
        ("LICENSE-LLVM.txt", "llvm"),
        ("LICENCE-MIT.txt", "mit"),
    ];

    fn fixture(directory: &Path, entries: &[(&str, &str)]) -> (Archive, PathBuf) {
        let zip = zip_with(directory, "dxc.zip", entries);
        let archive = Archive {
            name: "dxc_test".to_owned(),
            version: 1,
            url: "https://dxc.test/dxc.zip".to_owned(),
            sha256: sha256_of(&zip).unwrap(),
            size: fs::metadata(&zip).unwrap().len(),
            licenses: vec!["MIT".to_owned()],
        };
        (archive, zip)
    }

    fn run(archive: &Archive, target: &Path, source: &impl Download) -> Result<Vec<String>> {
        let mut lines = Vec::new();
        install(archive, target, source, &mut |line| lines.push(line))?;
        Ok(lines)
    }

    #[test]
    fn the_pinned_release_is_an_official_download_with_a_hash() {
        let release = release();

        assert!(
            release.url.starts_with(
                "https://github.com/microsoft/DirectXShaderCompiler/releases/download/"
            )
        );
        assert_eq!(release.sha256.len(), 64);
        assert!(release.size > 0);
    }

    #[test]
    fn install_puts_the_library_and_its_licenses_next_to_the_debug_binaries() {
        let directory = tempfile::tempdir().unwrap();
        let (archive, zip) = fixture(directory.path(), &ENTRIES);
        let target = directory.path().join("target");

        run(&archive, &target, &LocalFile(zip)).unwrap();

        let debug = target.join("debug");
        assert_eq!(
            fs::read_to_string(debug.join("dxcompiler.dll")).unwrap(),
            "compiler"
        );
        assert_eq!(
            fs::read_to_string(debug.join("dxcompiler-LICENSE-LLVM.txt")).unwrap(),
            "llvm"
        );
        assert_eq!(
            fs::read_to_string(debug.join("dxcompiler-LICENSE-MIT.txt")).unwrap(),
            "mit"
        );
    }

    #[test]
    fn install_also_serves_an_existing_release_folder() {
        let directory = tempfile::tempdir().unwrap();
        let (archive, zip) = fixture(directory.path(), &ENTRIES);
        let target = directory.path().join("target");
        fs::create_dir_all(target.join("release")).unwrap();

        run(&archive, &target, &LocalFile(zip)).unwrap();

        assert!(target.join("release").join("dxcompiler.dll").is_file());
    }

    #[test]
    fn a_second_install_does_not_download_again() {
        let directory = tempfile::tempdir().unwrap();
        let (archive, zip) = fixture(directory.path(), &ENTRIES);
        let target = directory.path().join("target");
        run(&archive, &target, &LocalFile(zip)).unwrap();

        let lines = run(&archive, &target, &Offline).unwrap();

        assert_eq!(lines[0], "dxc_test version 1 is current");
    }

    #[test]
    fn an_archive_without_the_library_is_an_error() {
        let directory = tempfile::tempdir().unwrap();
        let (archive, zip) = fixture(directory.path(), &[("LICENSE-LLVM.txt", "llvm")]);

        let error = run(&archive, &directory.path().join("target"), &LocalFile(zip)).unwrap_err();

        assert!(format!("{error:#}").contains("bin/x64/dxcompiler.dll"));
    }
}
