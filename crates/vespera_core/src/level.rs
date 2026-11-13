use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Level(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the level {0} is not from 1 to 99")]
pub struct LevelOutOfRange(pub u8);

impl Level {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(99);

    pub fn new(value: u8) -> Result<Self, LevelOutOfRange> {
        if (Self::MIN.0..=Self::MAX.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(LevelOutOfRange(value))
        }
    }

    #[must_use]
    pub fn get(self) -> u8 {
        self.0
    }

    #[must_use]
    pub fn next(self) -> Option<Self> {
        Self::new(self.0.checked_add(1)?).ok()
    }
}

impl TryFrom<u8> for Level {
    type Error = LevelOutOfRange;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Level> for u8 {
    fn from(level: Level) -> Self {
        level.get()
    }
}

#[cfg(test)]
mod tests {
    use super::{Level, LevelOutOfRange};

    #[test]
    fn levels_from_1_to_99_are_valid() {
        assert_eq!(Level::new(1), Ok(Level::MIN));
        assert_eq!(Level::new(99), Ok(Level::MAX));
        assert_eq!(Level::new(42).map(Level::get), Ok(42));
    }

    #[test]
    fn levels_outside_1_to_99_are_not_valid() {
        assert_eq!(Level::new(0), Err(LevelOutOfRange(0)));
        assert_eq!(Level::new(100), Err(LevelOutOfRange(100)));
    }

    #[test]
    fn the_next_level_stops_at_99() {
        assert_eq!(Level::MIN.next(), Level::new(2).ok());
        assert_eq!(Level::MAX.next(), None);
    }

    #[test]
    fn data_files_cannot_contain_a_level_outside_the_range() {
        assert_eq!(ron::from_str::<Level>("12").ok(), Level::new(12).ok());
        assert!(ron::from_str::<Level>("0").is_err());
    }

    #[test]
    fn the_error_names_the_level() {
        assert_eq!(
            LevelOutOfRange(120).to_string(),
            "the level 120 is not from 1 to 99"
        );
    }
}
