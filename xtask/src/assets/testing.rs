use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::Download;

pub struct LocalFile(pub PathBuf);

pub struct Offline;

impl Download for LocalFile {
    fn download(&self, _url: &str, destination: &Path) -> Result<()> {
        fs::copy(&self.0, destination)?;
        Ok(())
    }
}

impl Download for Offline {
    fn download(&self, url: &str, _destination: &Path) -> Result<()> {
        bail!("the test expected no download, but got {url}")
    }
}

pub fn zip_with(directory: &Path, file: &str, entries: &[(&str, &str)]) -> PathBuf {
    let path = directory.join(file);
    let mut writer = ZipWriter::new(File::create(&path).unwrap());
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, contents) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(contents.as_bytes()).unwrap();
    }
    writer.finish().unwrap();
    path
}
