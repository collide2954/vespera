use crate::finding::Finding;
use crate::repository::RepositoryFile;

const WORD_LIMIT: usize = 25;
const PREVIEW_WORDS: usize = 8;
const UNAPPROVED_WORDS: &[&str] = &[
    "allow", "allowed", "allows", "appear", "appears", "could", "create", "created", "creates",
    "e.g", "enable", "enabled", "enables", "ensure", "ensures", "etc", "get", "gets", "getting",
    "got", "i.e", "just", "may", "might", "need", "needed", "needs", "please", "provide",
    "provided", "provides", "really", "require", "required", "requires", "should", "very", "via",
    "would",
];

pub fn check(files: &[RepositoryFile]) -> Vec<Finding> {
    files
        .iter()
        .filter(|file| file.has_extension(&["md"]))
        .flat_map(check_file)
        .collect()
}

fn check_file(file: &RepositoryFile) -> Vec<Finding> {
    let Ok(text) = std::str::from_utf8(&file.bytes) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    let mut in_fence = false;
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        } else if !in_fence {
            let prose = prose(line);
            for cell in prose.split('|') {
                findings.extend(check_text(&file.path, index + 1, cell));
            }
        }
    }
    findings
}

fn check_text(path: &str, line: usize, text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for sentence in sentences(text) {
        let count = sentence.iter().filter(|word| is_word(word)).count();
        if count > WORD_LIMIT {
            let preview = sentence[..PREVIEW_WORDS.min(sentence.len())].join(" ");
            findings.push(Finding::at_line(
                path,
                line,
                format!(
                    "the sentence has {count} words, and the limit is {WORD_LIMIT}: {preview} ..."
                ),
            ));
        }
        for word in sentence {
            if UNAPPROVED_WORDS.contains(&normalized(word).as_str()) {
                findings.push(Finding::at_line(
                    path,
                    line,
                    format!("ASD-STE100 does not approve the word \"{}\"", shown(word)),
                ));
            }
        }
    }
    findings
}

fn prose(line: &str) -> String {
    let mut prose = String::with_capacity(line.len());
    let mut characters = line.chars().peekable();
    let mut in_code = false;
    while let Some(character) = characters.next() {
        match character {
            '`' => {
                in_code = !in_code;
                if in_code {
                    prose.push_str(" code ");
                }
            }
            _ if in_code => {}
            ']' if characters.peek() == Some(&'(') => {
                characters
                    .by_ref()
                    .take_while(|next| *next != ')')
                    .for_each(drop);
            }
            '[' | ']' => {}
            _ => prose.push(character),
        }
    }
    prose
}

fn sentences(text: &str) -> Vec<Vec<&str>> {
    let mut sentences = Vec::new();
    let mut current = Vec::new();
    for word in text.split_whitespace() {
        current.push(word);
        if word.ends_with(['.', '!', '?', ':']) {
            sentences.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        sentences.push(current);
    }
    sentences
}

fn is_word(word: &str) -> bool {
    word.chars().any(char::is_alphanumeric)
}

fn normalized(word: &str) -> String {
    word.trim_matches(|character: char| !character.is_alphanumeric())
        .to_lowercase()
}

fn shown(word: &str) -> &str {
    word.trim_matches(|character: char| !character.is_alphanumeric() && character != '.')
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::finding::Finding;
    use crate::repository::RepositoryFile;

    const NO_FINDINGS: [&str; 0] = [];

    fn findings_in(path: &str, contents: &str) -> Vec<String> {
        let files = [RepositoryFile {
            path: path.to_owned(),
            bytes: contents.as_bytes().to_vec(),
        }];
        check(&files).iter().map(Finding::to_string).collect()
    }

    fn sentence_of(words: usize) -> String {
        let mut sentence = vec!["word"; words].join(" ");
        sentence.push('.');
        sentence
    }

    #[test]
    fn a_sentence_with_more_than_25_words_is_a_finding() {
        let findings = findings_in("docs/a.md", &sentence_of(26));

        assert_eq!(
            findings,
            [
                "docs/a.md:1: the sentence has 26 words, and the limit is 25: word word word word word word word word ..."
            ]
        );
    }

    #[test]
    fn a_sentence_with_25_words_is_not_a_finding() {
        assert_eq!(findings_in("docs/a.md", &sentence_of(25)), NO_FINDINGS);
    }

    #[test]
    fn sentence_ends_divide_a_line_into_sentences() {
        let line = format!("{} {}", sentence_of(15), sentence_of(15));

        assert_eq!(findings_in("docs/a.md", &line), NO_FINDINGS);
    }

    #[test]
    fn table_cells_are_separate_texts() {
        let row = format!("| {} | {} |", sentence_of(15), sentence_of(15));

        assert_eq!(findings_in("docs/a.md", &row), NO_FINDINGS);
    }

    #[test]
    fn an_unapproved_word_is_a_finding() {
        assert_eq!(
            findings_in("docs/a.md", "You should run the tests."),
            ["docs/a.md:1: ASD-STE100 does not approve the word \"should\""]
        );
    }

    #[test]
    fn unapproved_words_match_without_case_or_punctuation() {
        assert_eq!(
            findings_in(
                "docs/a.md",
                "Fonts (e.g., titles) are text.\n\nShould we stop?"
            ),
            [
                "docs/a.md:1: ASD-STE100 does not approve the word \"e.g.\"",
                "docs/a.md:3: ASD-STE100 does not approve the word \"Should\"",
            ]
        );
    }

    #[test]
    fn words_that_contain_an_unapproved_word_are_permitted() {
        assert_eq!(
            findings_in("docs/a.md", "Add the entry to the allowlist."),
            NO_FINDINGS
        );
    }

    #[test]
    fn code_spans_are_not_checked() {
        assert_eq!(
            findings_in("docs/a.md", "Run `cargo test -- --should-panic` now."),
            NO_FINDINGS
        );
    }

    #[test]
    fn fenced_code_blocks_are_not_checked() {
        assert_eq!(
            findings_in("docs/a.md", "```text\nyou should\n```\n"),
            NO_FINDINGS
        );
    }

    #[test]
    fn link_targets_are_not_checked() {
        assert_eq!(
            findings_in("docs/a.md", "Read [the rules](https://example.com/should)."),
            NO_FINDINGS
        );
    }

    #[test]
    fn only_markdown_files_are_checked() {
        assert_eq!(findings_in("src/lib.rs", "You should run it."), NO_FINDINGS);
    }
}
