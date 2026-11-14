use bevy::math::Vec3;
use bevy::prelude::Entity;

#[must_use]
pub fn nearest_in_reach(
    origin: Vec3,
    candidates: impl IntoIterator<Item = (Entity, Vec3)>,
    reach: f32,
) -> Option<Entity> {
    candidates
        .into_iter()
        .map(|(entity, position)| (entity, origin.distance(position)))
        .filter(|(_, distance)| *distance <= reach)
        .min_by(|(_, first), (_, second)| first.total_cmp(second))
        .map(|(entity, _)| entity)
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;
    use bevy::prelude::Entity;

    use super::nearest_in_reach;

    fn entity(index: u32) -> Entity {
        Entity::from_raw_u32(index).unwrap()
    }

    #[test]
    fn the_nearest_candidate_in_reach_is_the_target() {
        let candidates = [
            (entity(1), Vec3::new(2.0, 0.0, 0.0)),
            (entity(2), Vec3::new(1.0, 0.0, 0.0)),
        ];

        assert_eq!(
            nearest_in_reach(Vec3::ZERO, candidates, 2.5),
            Some(entity(2))
        );
    }

    #[test]
    fn candidates_out_of_reach_are_not_targets() {
        let candidates = [(entity(1), Vec3::new(5.0, 0.0, 0.0))];

        assert_eq!(nearest_in_reach(Vec3::ZERO, candidates, 2.5), None);
    }

    #[test]
    fn no_candidates_give_no_target() {
        assert_eq!(nearest_in_reach(Vec3::ZERO, [], 2.5), None);
    }
}
