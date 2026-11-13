use vespera_core::Level;

const FIRST_STEP: f64 = 80.0;
const GROWTH: f64 = 1.1;

#[must_use]
pub fn experience_for_level(level: Level) -> u32 {
    (1..level.get()).map(step).sum()
}

#[must_use]
pub fn level_for_experience(experience: u32) -> Level {
    let mut level = Level::MIN;
    let mut threshold = 0;
    while let Some(next) = level.next() {
        threshold += step(level.get());
        if experience < threshold {
            break;
        }
        level = next;
    }
    level
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "each step is positive and less than one million"
)]
fn step(level: u8) -> u32 {
    (FIRST_STEP * GROWTH.powi(i32::from(level) - 1)).round() as u32
}

#[cfg(test)]
mod tests {
    use vespera_core::Level;

    use super::{experience_for_level, level_for_experience};

    fn level(value: u8) -> Level {
        Level::new(value).unwrap()
    }

    #[test]
    fn level_1_needs_no_experience() {
        assert_eq!(experience_for_level(Level::MIN), 0);
    }

    #[test]
    fn each_level_needs_ten_percent_more_than_the_level_before() {
        assert_eq!(experience_for_level(level(2)), 80);
        assert_eq!(experience_for_level(level(3)), 168);
        assert_eq!(experience_for_level(level(4)), 265);
    }

    #[test]
    fn level_99_needs_approximately_9_1_million_experience() {
        assert_eq!(experience_for_level(Level::MAX), 9_110_348);
    }

    #[test]
    fn level_92_is_approximately_half_of_level_99() {
        assert_eq!(experience_for_level(level(92)), 4_674_659);
    }

    #[test]
    fn a_level_starts_at_its_experience_threshold() {
        assert_eq!(level_for_experience(0), Level::MIN);
        assert_eq!(level_for_experience(79), Level::MIN);
        assert_eq!(level_for_experience(80), level(2));
        assert_eq!(level_for_experience(u32::MAX), Level::MAX);
    }

    #[test]
    fn levels_and_thresholds_agree_for_all_levels() {
        for value in 2..=Level::MAX.get() {
            let threshold = experience_for_level(level(value));

            assert_eq!(level_for_experience(threshold), level(value));
            assert_eq!(level_for_experience(threshold - 1), level(value - 1));
        }
    }
}
