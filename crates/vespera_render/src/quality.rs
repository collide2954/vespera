use bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::anti_alias::fxaa::Fxaa;
use bevy::light::{CascadeShadowConfig, CascadeShadowConfigBuilder, ShadowFilteringMethod};
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quality {
    pub msaa: Msaa,
    pub shadow_filter: ShadowFilteringMethod,
    pub shadow_cascades: usize,
    pub shadow_map_size: usize,
    pub first_cascade_distance: f32,
    pub shadow_distance: f32,
    pub min_render_scale: f32,
    pub max_render_scale: f32,
    pub indirect_drawing: bool,
    pub gpu_light_clustering: bool,
}

impl Quality {
    pub const LOW: Self = Self {
        msaa: Msaa::Off,
        shadow_filter: ShadowFilteringMethod::Hardware2x2,
        shadow_cascades: 2,
        shadow_map_size: 1024,
        first_cascade_distance: 15.0,
        shadow_distance: 60.0,
        min_render_scale: 0.67,
        max_render_scale: 1.0,
        indirect_drawing: false,
        gpu_light_clustering: false,
    };

    #[must_use]
    pub fn scene_camera(&self) -> impl Bundle {
        (
            self.msaa,
            Fxaa::default(),
            ContrastAdaptiveSharpening::default(),
            self.shadow_filter,
        )
    }

    #[must_use]
    pub fn screen_camera(&self) -> impl Bundle {
        self.msaa
    }

    #[must_use]
    pub fn sun_shadows(&self) -> CascadeShadowConfig {
        CascadeShadowConfigBuilder {
            num_cascades: self.shadow_cascades,
            first_cascade_far_bound: self.first_cascade_distance,
            maximum_distance: self.shadow_distance,
            ..default()
        }
        .build()
    }
}

#[cfg(test)]
mod tests {
    use bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
    use bevy::anti_alias::fxaa::Fxaa;
    use bevy::light::ShadowFilteringMethod;
    use bevy::prelude::*;

    use super::Quality;

    #[test]
    fn the_low_preset_follows_the_design() {
        assert_eq!(Quality::LOW.msaa, Msaa::Off);
        assert_eq!(Quality::LOW.shadow_cascades, 2);
        assert_eq!(Quality::LOW.shadow_map_size, 1024);
        assert!((Quality::LOW.min_render_scale - 0.67).abs() < 1e-6);
        assert!((Quality::LOW.max_render_scale - 1.0).abs() < 1e-6);
        assert!(!Quality::LOW.indirect_drawing);
        assert!(!Quality::LOW.gpu_light_clustering);
    }

    #[test]
    fn a_low_scene_camera_uses_fxaa_and_sharpening_without_msaa() {
        let mut world = World::new();

        let camera = world.spawn(Quality::LOW.scene_camera()).id();

        assert_eq!(world.get::<Msaa>(camera), Some(&Msaa::Off));
        assert!(world.get::<Fxaa>(camera).is_some_and(|fxaa| fxaa.enabled));
        assert!(
            world
                .get::<ContrastAdaptiveSharpening>(camera)
                .is_some_and(|sharpening| sharpening.enabled)
        );
    }

    #[test]
    fn a_low_screen_camera_only_scales_the_scene_up() {
        let mut world = World::new();

        let camera = world.spawn(Quality::LOW.screen_camera()).id();

        assert_eq!(world.get::<Msaa>(camera), Some(&Msaa::Off));
        assert!(world.get::<ContrastAdaptiveSharpening>(camera).is_none());
        assert!(world.get::<Fxaa>(camera).is_none());
    }

    #[test]
    fn a_low_scene_camera_uses_the_hardware_2x2_shadow_filter() {
        let mut world = World::new();

        let camera = world.spawn(Quality::LOW.scene_camera()).id();

        assert_eq!(
            world.get::<ShadowFilteringMethod>(camera),
            Some(&ShadowFilteringMethod::Hardware2x2)
        );
    }

    #[test]
    fn the_low_sun_has_two_cascades_that_end_at_the_shadow_distance() {
        let shadows = Quality::LOW.sun_shadows();

        assert_eq!(shadows.bounds.len(), 2);
        assert!((shadows.bounds[0] - 15.0).abs() < 1e-3);
        assert!((shadows.bounds[1] - 60.0).abs() < 1e-3);
    }
}
