use bevy::prelude::{Component, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Position(pub Vec3);
