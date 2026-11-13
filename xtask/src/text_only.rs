use crate::finding::Finding;
use crate::repository::RepositoryFile;

pub fn check(files: &[RepositoryFile]) -> Vec<Finding> {
    files
        .iter()
        .filter(|file| !is_text(&file.bytes))
        .map(|file| Finding::in_file(&file.path, "the file is not UTF-8 text"))
        .collect()
}

fn is_text(bytes: &[u8]) -> bool {
    !bytes.contains(&0) && std::str::from_utf8(bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::{check, is_text};
    use crate::finding::Finding;
    use crate::repository::RepositoryFile;

    #[test]
    fn utf8_text_is_text() {
        assert!(is_text("Größe × 2 – starlight".as_bytes()));
    }

    #[test]
    fn a_nul_byte_makes_a_file_binary() {
        assert!(!is_text(b"abc\0def"));
    }

    #[test]
    fn invalid_utf8_makes_a_file_binary() {
        assert!(!is_text(&[0xc3, 0x28]));
    }

    #[test]
    fn the_check_reports_only_binary_files() {
        let files = [
            RepositoryFile {
                path: "README.md".to_owned(),
                bytes: b"# Vespera".to_vec(),
            },
            RepositoryFile {
                path: "image.png".to_owned(),
                bytes: vec![0x89, b'P', b'N', b'G', 0x00],
            },
        ];

        assert_eq!(
            check(&files),
            [Finding::in_file("image.png", "the file is not UTF-8 text")]
        );
    }
}
