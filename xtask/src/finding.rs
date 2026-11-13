use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub message: String,
}

impl Finding {
    pub fn in_file(path: &str, message: impl Into<String>) -> Self {
        Self {
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::Finding;

    #[test]
    fn a_finding_shows_the_path_and_the_message() {
        let finding = Finding::in_file("image.png", "the file is not UTF-8 text");

        assert_eq!(finding.to_string(), "image.png: the file is not UTF-8 text");
    }
}
