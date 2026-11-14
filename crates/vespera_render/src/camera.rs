use bevy::math::Vec3;
use bevy::prelude::Transform;

pub const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 6.0, 9.0);

#[must_use]
pub fn follow_camera(target: Vec3) -> Transform {
    Transform::from_translation(target + CAMERA_OFFSET).looking_at(target + Vec3::Y, Vec3::Y)
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;

    use super::{CAMERA_OFFSET, follow_camera};

    #[test]
    fn the_camera_is_behind_and_above_the_target() {
        let target = Vec3::new(3.0, 0.0, -2.0);

        assert_eq!(follow_camera(target).translation, target + CAMERA_OFFSET);
    }

    #[test]
    fn the_camera_looks_at_the_target() {
        let target = Vec3::new(3.0, 0.0, -2.0);
        let camera = follow_camera(target);

        let to_target = (target + Vec3::Y - camera.translation).normalize();
        assert!(camera.forward().dot(to_target) > 0.999);
    }
}
