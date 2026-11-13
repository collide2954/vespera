use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub line: Option<usize>,
    pub message: String,
}

impl Finding {
    pub fn in_file(path: &str, message: impl Into<String>) -> Self {
        Self {
            path: path.to_owned(),
            line: None,
            message: message.into(),
        }
    }

    pub fn at_line(path: &str, line: usize, message: impl Into<String>) -> Self {
        Self {
            path: path.to_owned(),
            line: Some(line),
            message: message.into(),
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(formatter, "{}:{line}: {}", self.path, self.message),
            None => write!(formatter, "{}: {}", self.path, self.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Finding;

    #[test]
    fn a_finding_in_a_file_shows_the_path_and_the_message() {
        let finding = Finding::in_file("image.png", "the file is not UTF-8 text");

        assert_eq!(finding.to_string(), "image.png: the file is not UTF-8 text");
    }

    #[test]
    fn a_finding_at_a_line_shows_the_path_and_the_line() {
        let finding = Finding::at_line("src/lib.rs", 7, "this comment is not permitted");

        assert_eq!(
            finding.to_string(),
            "src/lib.rs:7: this comment is not permitted"
        );
    }
}
