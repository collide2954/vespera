pub const MAX_LEVEL: u8 = 99;

const FIRST_STEP: f64 = 80.0;
const GROWTH: f64 = 1.1;

#[must_use]
pub fn experience_for_level(level: u8) -> u32 {
    (1..level.clamp(1, MAX_LEVEL)).map(step).sum()
}

#[must_use]
pub fn level_for_experience(experience: u32) -> u8 {
    let mut level = 1;
    let mut threshold = 0;
    while level < MAX_LEVEL {
        threshold += step(level);
        if experience < threshold {
            break;
        }
        level += 1;
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
    use super::{MAX_LEVEL, experience_for_level, level_for_experience};

    #[test]
    fn level_1_needs_no_experience() {
        assert_eq!(experience_for_level(1), 0);
    }

    #[test]
    fn each_level_needs_ten_percent_more_than_the_level_before() {
        assert_eq!(experience_for_level(2), 80);
        assert_eq!(experience_for_level(3), 168);
        assert_eq!(experience_for_level(4), 265);
    }

    #[test]
    fn level_99_needs_approximately_9_1_million_experience() {
        assert_eq!(experience_for_level(MAX_LEVEL), 9_110_348);
    }

    #[test]
    fn level_92_is_approximately_half_of_level_99() {
        assert_eq!(experience_for_level(92), 4_674_659);
    }

    #[test]
    fn levels_outside_the_range_are_clamped() {
        assert_eq!(experience_for_level(0), 0);
        assert_eq!(experience_for_level(120), experience_for_level(MAX_LEVEL));
    }

    #[test]
    fn a_level_starts_at_its_experience_threshold() {
        assert_eq!(level_for_experience(0), 1);
        assert_eq!(level_for_experience(79), 1);
        assert_eq!(level_for_experience(80), 2);
        assert_eq!(level_for_experience(u32::MAX), MAX_LEVEL);
    }

    #[test]
    fn levels_and_thresholds_agree_for_all_levels() {
        for level in 2..=MAX_LEVEL {
            let threshold = experience_for_level(level);

            assert_eq!(level_for_experience(threshold), level);
            assert_eq!(level_for_experience(threshold - 1), level - 1);
        }
    }
}
