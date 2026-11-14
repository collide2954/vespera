use bevy::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveKey {
    Forward,
    Back,
    Left,
    Right,
}

impl MoveKey {
    fn direction(self) -> Vec2 {
        match self {
            Self::Forward => Vec2::NEG_Y,
            Self::Back => Vec2::Y,
            Self::Left => Vec2::NEG_X,
            Self::Right => Vec2::X,
        }
    }
}

#[must_use]
pub fn move_direction(held: impl IntoIterator<Item = MoveKey>) -> Vec2 {
    held.into_iter()
        .map(MoveKey::direction)
        .sum::<Vec2>()
        .normalize_or_zero()
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::{MoveKey, move_direction};

    #[test]
    fn no_keys_give_no_direction() {
        assert_eq!(move_direction([]), Vec2::ZERO);
    }

    #[test]
    fn forward_moves_away_from_the_camera() {
        assert_eq!(move_direction([MoveKey::Forward]), Vec2::new(0.0, -1.0));
    }

    #[test]
    fn two_keys_give_a_diagonal_of_length_one() {
        let direction = move_direction([MoveKey::Forward, MoveKey::Right]);

        assert!((direction.length() - 1.0).abs() < 1e-6);
        assert!(direction.x > 0.0 && direction.y < 0.0);
    }

    #[test]
    fn opposite_keys_cancel_each_other() {
        let all = [
            MoveKey::Forward,
            MoveKey::Back,
            MoveKey::Left,
            MoveKey::Right,
        ];

        assert_eq!(move_direction(all), Vec2::ZERO);
    }
}
