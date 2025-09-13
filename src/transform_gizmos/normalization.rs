//! Normalization module. See [`Normalize3d`].

use bevy::{prelude::*, render::camera::Camera};

use crate::{GizmoCamera, TransformGizmoSettings, TransformGizmoSystems};

/// Plugin for normalizing the size of a 3d object in the camera's view.
pub struct Ui3dNormalizationPlugin;

impl Plugin for Ui3dNormalizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            normalize
                .in_set(TransformGizmoSystems::Normalize)
                .after(TransformSystem::TransformPropagate)
                .after(TransformGizmoSystems::Place)
                .run_if(|settings: Res<TransformGizmoSettings>| settings.enabled),
        );
    }
}

/// Struct that marks entities with meshes that should be scaled relative to the camera.
#[derive(Component, Debug)]
pub struct Normalize3d {
    /// Length of the object in world space units
    pub size_in_world: f32,
    /// Desired length of the object in pixels
    pub desired_pixel_size: f32,
}

impl Normalize3d {
    /// Construct a new [`Normalize3d`].
    pub fn new(size_in_world: f32, desired_pixel_size: f32) -> Self {
        Normalize3d {
            size_in_world,
            desired_pixel_size,
        }
    }
}

/// The system that performs size normalization based on the [`Normalize3d`] component.
pub fn normalize(
    mut query: ParamSet<(
        Query<(&GlobalTransform, &Camera), With<GizmoCamera>>,
        Query<(&mut Transform, &mut GlobalTransform, &Normalize3d)>,
    )>,
) {
    // Find the first active camera with GizmoCamera marker
    let (camera_position, camera) = if let Some((camera_position, camera)) =
        query.p0().iter().find(|(_, camera)| camera.is_active)
    {
        (camera_position.to_owned(), camera.to_owned())
    } else if let Some((camera_position, camera)) = query.p0().iter().next() {
        // Fallback to first camera if no active camera found
        (camera_position.to_owned(), camera.to_owned())
    } else {
        // No cameras found, skip normalization
        return;
    };
    let view = camera_position.compute_matrix().inverse();

    for (mut transform, mut global_transform, normalize) in query.p1().iter_mut() {
        let distance = view.transform_point3(global_transform.translation()).z;
        let gt = global_transform.compute_transform();
        let Ok(pixel_end) = Camera::world_to_viewport(
            &camera,
            &GlobalTransform::default(),
            Vec3::new(normalize.size_in_world * gt.scale.x, 0.0, distance),
        ) else {
            continue;
        };
        let Ok(pixel_root) = Camera::world_to_viewport(
            &camera,
            &GlobalTransform::default(),
            Vec3::new(0.0, 0.0, distance),
        ) else {
            continue;
        };
        let actual_pixel_size = pixel_root.distance(pixel_end);
        let required_scale = normalize.desired_pixel_size / actual_pixel_size;
        transform.scale = gt.scale * Vec3::splat(required_scale);
        *global_transform = (*transform).into();
    }
}
