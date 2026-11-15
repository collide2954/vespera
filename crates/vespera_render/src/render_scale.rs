use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::window::PrimaryWindow;

use crate::Quality;
use crate::scale_controller::{ScaleController, adjust_render_scale};

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct RenderScale(pub f32);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SceneView;

pub struct RenderScalePlugin;

impl Plugin for RenderScalePlugin {
    fn build(&self, app: &mut App) {
        let controller =
            ScaleController::new(Quality::LOW.min_render_scale, Quality::LOW.max_render_scale);
        app.insert_resource(RenderScale(controller.scale()))
            .insert_resource(controller)
            .add_systems(Update, (adjust_render_scale, fit_scene_to_window).chain());
    }
}

#[must_use]
pub fn scene_size(window: UVec2, scale: f32) -> UVec2 {
    (window.as_vec2() * scale)
        .round()
        .as_uvec2()
        .max(UVec2::ONE)
}

#[must_use]
pub fn scene_image() -> Image {
    let mut image = Image::new_target_texture(1, 1, TextureFormat::Rgba8UnormSrgb, None);
    image.data = None;
    image.copy_on_resize = false;
    image
}

fn fit_scene_to_window(
    scale: Res<RenderScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut views: Query<&mut Sprite, With<SceneView>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let size = scene_size(window.physical_size(), scale.0);
    let screen = Some(window.size());
    for mut view in &mut views {
        if view.custom_size != screen {
            view.custom_size = screen;
        }
        if images
            .get(&view.image)
            .is_some_and(|image| image.size() != size)
            && let Some(mut image) = images.get_mut(&view.image)
        {
            image.resize(Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::render::render_resource::TextureUsages;
    use bevy::window::{PrimaryWindow, WindowResolution};

    use super::{RenderScale, RenderScalePlugin, SceneView, scene_image, scene_size};

    fn app(scale: f32) -> (App, Handle<Image>) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RenderScalePlugin))
            .init_resource::<Assets<Image>>()
            .insert_resource(RenderScale(scale));
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(scene_image());
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(1920, 1080),
                ..default()
            },
            PrimaryWindow,
        ));
        app.world_mut()
            .spawn((Sprite::from_image(image.clone()), SceneView));
        (app, image)
    }

    #[test]
    fn the_scene_size_is_the_window_size_times_the_render_scale() {
        assert_eq!(
            scene_size(UVec2::new(1920, 1080), 0.67),
            UVec2::new(1286, 724)
        );
    }

    #[test]
    fn the_scene_is_never_smaller_than_one_pixel() {
        assert_eq!(scene_size(UVec2::ZERO, 0.67), UVec2::ONE);
    }

    #[test]
    fn the_scene_image_is_a_render_target_that_keeps_no_copies() {
        let image = scene_image();

        assert!(image.data.is_none());
        assert!(!image.copy_on_resize);
        assert!(
            image
                .texture_descriptor
                .usage
                .contains(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
        );
    }

    #[test]
    fn the_scene_image_follows_the_window_at_the_render_scale() {
        let (mut app, image) = app(0.5);

        app.update();

        let size = app
            .world()
            .resource::<Assets<Image>>()
            .get(&image)
            .map(Image::size);
        assert_eq!(size, Some(UVec2::new(960, 540)));
    }

    #[test]
    fn the_scene_view_covers_the_window() {
        let (mut app, _) = app(0.5);

        app.update();

        let world = app.world_mut();
        let view = world
            .query_filtered::<&Sprite, With<SceneView>>()
            .single(world)
            .expect("one scene view");
        assert_eq!(view.custom_size, Some(Vec2::new(1920.0, 1080.0)));
    }
}
