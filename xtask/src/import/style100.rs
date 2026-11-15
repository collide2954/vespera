use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use zip::ZipArchive;

use super::clip_list::Capture;
use super::frame_cuts::FrameCuts;

pub const ARCHIVE: &str = "100style/100STYLE.zip";

const FRAME_CUTS: &str = "100STYLE/Frame_Cuts.csv";

pub struct Style100 {
    archive: ZipArchive<BufReader<File>>,
}

impl Style100 {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("cannot open the 100STYLE archive {}", path.display()))?;
        let archive = ZipArchive::new(BufReader::new(file))
            .with_context(|| format!("{} is not a zip archive", path.display()))?;
        Ok(Self { archive })
    }

    pub fn frame_cuts(&mut self) -> Result<FrameCuts> {
        let mut text = String::new();
        self.archive
            .by_name(FRAME_CUTS)
            .with_context(|| format!("the 100STYLE archive has no {FRAME_CUTS}"))?
            .read_to_string(&mut text)
            .with_context(|| format!("cannot read {FRAME_CUTS}"))?;
        FrameCuts::parse(&text)
    }

    pub fn extract(&mut self, style: &str, capture: Capture, folder: &Path) -> Result<PathBuf> {
        let file_name = format!("{style}_{}.bvh", capture.code());
        let entry_name = format!("100STYLE/{style}/{file_name}");
        let mut entry = self
            .archive
            .by_name(&entry_name)
            .with_context(|| format!("the 100STYLE archive has no {entry_name}"))?;
        fs::create_dir_all(folder)
            .with_context(|| format!("cannot create {}", folder.display()))?;
        let path = folder.join(file_name);
        let mut output =
            File::create(&path).with_context(|| format!("cannot create {}", path.display()))?;
        io::copy(&mut entry, &mut output)
            .with_context(|| format!("cannot extract {entry_name}"))?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Style100;
    use crate::assets::testing::zip_with;
    use crate::import::clip_list::Capture;

    const CUTS: &str = "STYLE_NAME,FW_START,FW_STOP\nNeutral,327,7445\n";

    #[test]
    fn the_frame_cuts_come_from_the_archive() {
        let directory = tempfile::tempdir().unwrap();
        let zip = zip_with(
            directory.path(),
            "100STYLE.zip",
            &[("100STYLE/Frame_Cuts.csv", CUTS)],
        );

        let cuts = Style100::open(&zip).unwrap().frame_cuts().unwrap();

        assert_eq!(
            cuts.range("Neutral", Capture::ForwardWalk).unwrap(),
            (327, 7445)
        );
    }

    #[test]
    fn a_capture_is_extracted_to_a_file_with_its_style_and_code() {
        let directory = tempfile::tempdir().unwrap();
        let zip = zip_with(
            directory.path(),
            "100STYLE.zip",
            &[("100STYLE/Neutral/Neutral_FW.bvh", "HIERARCHY")],
        );
        let folder = directory.path().join("captures");

        let path = Style100::open(&zip)
            .unwrap()
            .extract("Neutral", Capture::ForwardWalk, &folder)
            .unwrap();

        assert_eq!(path, folder.join("Neutral_FW.bvh"));
        assert_eq!(fs::read_to_string(path).unwrap(), "HIERARCHY");
    }

    #[test]
    fn a_missing_capture_is_an_error_with_its_path_in_the_archive() {
        let directory = tempfile::tempdir().unwrap();
        let zip = zip_with(
            directory.path(),
            "100STYLE.zip",
            &[("100STYLE/Frame_Cuts.csv", CUTS)],
        );

        let error = Style100::open(&zip)
            .unwrap()
            .extract("Neutral", Capture::ForwardRun, directory.path())
            .unwrap_err();

        assert!(format!("{error:#}").contains("100STYLE/Neutral/Neutral_FR.bvh"));
    }

    #[test]
    fn a_missing_archive_is_an_error_with_its_path() {
        let directory = tempfile::tempdir().unwrap();
        let missing = directory.path().join("100STYLE.zip");

        let Err(error) = Style100::open(&missing) else {
            panic!("a missing archive must be an error");
        };

        assert!(format!("{error:#}").contains(&missing.display().to_string()));
    }
}
