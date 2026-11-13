use anyhow::{Context, Result};
use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};
use serde::Deserialize;

use crate::finding::Finding;
use crate::repository::RepositoryFile;

pub const ALLOWLIST_PATH: &str = "comment-allowlist.toml";

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Allowlist {
    #[serde(default, rename = "comment")]
    entries: Vec<AllowedComment>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AllowedComment {
    path: String,
    text: String,
    reason: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Comment {
    line: usize,
    text: String,
}

impl Allowlist {
    pub fn parse(text: &str) -> Result<Self> {
        toml::from_str(text).with_context(|| format!("{ALLOWLIST_PATH} is not valid"))
    }

    pub fn from_files(files: &[RepositoryFile]) -> Result<Self> {
        files
            .iter()
            .find(|file| file.path == ALLOWLIST_PATH)
            .map_or_else(
                || Ok(Self::default()),
                |file| Self::parse(&String::from_utf8_lossy(&file.bytes)),
            )
    }

    fn position(&self, path: &str, comment: &Comment) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| entry.path == path && entry.text == comment.text)
    }
}

pub fn check(files: &[RepositoryFile], allowlist: &Allowlist) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut used = vec![false; allowlist.entries.len()];
    for file in files {
        for comment in comments_in(file) {
            match allowlist.position(&file.path, &comment) {
                Some(index) => used[index] = true,
                None => findings.push(Finding::at_line(
                    &file.path,
                    comment.line,
                    format!("this comment is not permitted: {}", comment.text),
                )),
            }
        }
    }
    for (entry, was_used) in allowlist.entries.iter().zip(used) {
        if entry.reason.trim().is_empty() {
            findings.push(Finding::in_file(
                ALLOWLIST_PATH,
                format!("the entry for {} has no reason", entry.path),
            ));
        }
        if !was_used {
            findings.push(Finding::in_file(
                ALLOWLIST_PATH,
                format!(
                    "the entry for {} matches no comment: {}",
                    entry.path, entry.text
                ),
            ));
        }
    }
    findings
}

fn comments_in(file: &RepositoryFile) -> Vec<Comment> {
    let Ok(source) = std::str::from_utf8(&file.bytes) else {
        return Vec::new();
    };
    match file.path.rsplit_once('.').map(|(_, extension)| extension) {
        Some("rs") => rust_comments(source),
        Some("wesl" | "wgsl") => shader_comments(source),
        _ => Vec::new(),
    }
}

fn rust_comments(source: &str) -> Vec<Comment> {
    let mut comments = Vec::new();
    let mut start = 0;
    for token in tokenize(source, FrontmatterAllowed::No) {
        let end = start + token.len as usize;
        if matches!(
            token.kind,
            TokenKind::LineComment { doc_style: None }
                | TokenKind::BlockComment {
                    doc_style: None,
                    ..
                }
        ) {
            comments.push(Comment {
                line: line_at(source, start),
                text: source[start..end].trim_end().to_owned(),
            });
        }
        start = end;
    }
    comments
}

fn shader_comments(source: &str) -> Vec<Comment> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let start = [line.find("//"), line.find("/*")]
                .into_iter()
                .flatten()
                .min()?;
            Some(Comment {
                line: index + 1,
                text: line[start..].trim_end().to_owned(),
            })
        })
        .collect()
}

fn line_at(source: &str, offset: usize) -> usize {
    source[..offset].matches('\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::{Allowlist, check};
    use crate::finding::Finding;
    use crate::repository::RepositoryFile;

    fn file(path: &str, contents: &str) -> RepositoryFile {
        RepositoryFile {
            path: path.to_owned(),
            bytes: contents.as_bytes().to_vec(),
        }
    }

    const NO_FINDINGS: [&str; 0] = [];

    fn findings(files: &[RepositoryFile], allowlist: &Allowlist) -> Vec<String> {
        check(files, allowlist)
            .iter()
            .map(Finding::to_string)
            .collect()
    }

    fn allowlist_entry(path: &str, text: &str, reason: &str) -> Allowlist {
        Allowlist::parse(&format!(
            "[[comment]]\npath = {path:?}\ntext = {text:?}\nreason = {reason:?}\n"
        ))
        .unwrap()
    }

    #[test]
    fn a_line_comment_in_rust_is_a_finding() {
        let files = [file("src/lib.rs", "fn a() {}\n// a note\n")];

        assert_eq!(
            findings(&files, &Allowlist::default()),
            ["src/lib.rs:2: this comment is not permitted: // a note"]
        );
    }

    #[test]
    fn a_block_comment_in_rust_is_a_finding() {
        let files = [file("src/lib.rs", "/* a note */ fn a() {}\n")];

        assert_eq!(
            findings(&files, &Allowlist::default()),
            ["src/lib.rs:1: this comment is not permitted: /* a note */"]
        );
    }

    #[test]
    fn four_slashes_make_a_normal_comment() {
        let files = [file("src/lib.rs", "//// a note\nfn a() {}\n")];

        assert_eq!(
            findings(&files, &Allowlist::default()),
            ["src/lib.rs:1: this comment is not permitted: //// a note"]
        );
    }

    #[test]
    fn rustdoc_comments_are_permitted() {
        let source = "//! The crate.\n\n/// A function.\nfn a() {}\n\n/** A block. */\nfn b() {}\n";

        assert_eq!(
            findings(&[file("src/lib.rs", source)], &Allowlist::default()),
            NO_FINDINGS
        );
    }

    #[test]
    fn comment_markers_in_strings_and_characters_are_not_comments() {
        let source = "const URL: &str = \"https://example.com\";\nconst RAW: &str = r#\"/* not */\"#;\nconst SLASH: char = '/';\n";

        assert_eq!(
            findings(&[file("src/lib.rs", source)], &Allowlist::default()),
            NO_FINDINGS
        );
    }

    #[test]
    fn a_line_comment_in_a_shader_is_a_finding() {
        let files = [file(
            "shaders/sky.wesl",
            "fn sky() {}\nfn sun() {} // a note\n",
        )];

        assert_eq!(
            findings(&files, &Allowlist::default()),
            ["shaders/sky.wesl:2: this comment is not permitted: // a note"]
        );
    }

    #[test]
    fn a_block_comment_in_a_shader_is_a_finding() {
        let files = [file("shaders/sky.wgsl", "/* a note */\nfn sky() {}\n")];

        assert_eq!(
            findings(&files, &Allowlist::default()),
            ["shaders/sky.wgsl:1: this comment is not permitted: /* a note */"]
        );
    }

    #[test]
    fn other_files_are_not_checked() {
        let files = [file("docs/notes.md", "// not code\n")];

        assert_eq!(findings(&files, &Allowlist::default()), NO_FINDINGS);
    }

    #[test]
    fn a_permitted_comment_is_not_a_finding() {
        let allowlist = allowlist_entry("src/lib.rs", "// a note", "A link to an upstream bug.");
        let files = [file("src/lib.rs", "// a note\nfn a() {}\n")];

        assert_eq!(findings(&files, &allowlist), NO_FINDINGS);
    }

    #[test]
    fn an_entry_without_a_reason_is_a_finding() {
        let allowlist = allowlist_entry("src/lib.rs", "// a note", " ");
        let files = [file("src/lib.rs", "// a note\nfn a() {}\n")];

        assert_eq!(
            findings(&files, &allowlist),
            ["comment-allowlist.toml: the entry for src/lib.rs has no reason"]
        );
    }

    #[test]
    fn an_entry_that_matches_no_comment_is_a_finding() {
        let allowlist = allowlist_entry("src/lib.rs", "// old note", "A reason.");
        let files = [file("src/lib.rs", "fn a() {}\n")];

        assert_eq!(
            findings(&files, &allowlist),
            ["comment-allowlist.toml: the entry for src/lib.rs matches no comment: // old note"]
        );
    }

    #[test]
    fn an_allowlist_with_an_unknown_field_is_an_error() {
        let text = "[[comment]]\npath = \"a\"\ntext = \"b\"\nreason = \"c\"\nextra = 1\n";

        assert!(Allowlist::parse(text).is_err());
    }

    #[test]
    fn no_allowlist_file_gives_an_empty_allowlist() {
        assert!(Allowlist::from_files(&[]).unwrap().entries.is_empty());
    }

    #[test]
    fn the_allowlist_comes_from_the_allowlist_file() {
        let files = [file(
            "comment-allowlist.toml",
            "[[comment]]\npath = \"src/lib.rs\"\ntext = \"// a note\"\nreason = \"A reason.\"\n",
        )];

        assert_eq!(Allowlist::from_files(&files).unwrap().entries.len(), 1);
    }
}
