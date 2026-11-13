use std::collections::BTreeMap;

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};
use vespera_core::Level;

use crate::experience::level_for_experience;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Skill {
    Woodcutting,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Skills {
    experience: BTreeMap<Skill, u32>,
}

impl Skills {
    #[must_use]
    pub fn experience(&self, skill: Skill) -> u32 {
        self.experience.get(&skill).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn level(&self, skill: Skill) -> Level {
        level_for_experience(self.experience(skill))
    }

    pub fn add_experience(&mut self, skill: Skill, amount: u32) {
        let experience = self.experience.entry(skill).or_insert(0);
        *experience = experience.saturating_add(amount);
    }
}

#[cfg(test)]
mod tests {
    use vespera_core::Level;

    use super::{Skill, Skills};

    #[test]
    fn a_new_character_has_level_1_and_no_experience() {
        let skills = Skills::default();

        assert_eq!(skills.experience(Skill::Woodcutting), 0);
        assert_eq!(skills.level(Skill::Woodcutting), Level::MIN);
    }

    #[test]
    fn experience_increases_the_level() {
        let mut skills = Skills::default();

        skills.add_experience(Skill::Woodcutting, 50);
        skills.add_experience(Skill::Woodcutting, 30);

        assert_eq!(skills.experience(Skill::Woodcutting), 80);
        assert_eq!(skills.level(Skill::Woodcutting), Level::new(2).unwrap());
    }

    #[test]
    fn experience_stops_at_its_maximum() {
        let mut skills = Skills::default();

        skills.add_experience(Skill::Woodcutting, u32::MAX);
        skills.add_experience(Skill::Woodcutting, 1);

        assert_eq!(skills.experience(Skill::Woodcutting), u32::MAX);
        assert_eq!(skills.level(Skill::Woodcutting), Level::MAX);
    }
}
