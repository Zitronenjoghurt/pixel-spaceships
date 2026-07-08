use crate::plugins::GameConfig;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

/// Marks the entity the camera should track. The camera follows whatever holds
/// this, so it stays decoupled from the ship / world specifics.
#[derive(Component)]
pub struct CameraTarget;

/// The camera-follow step. Anything positioned relative to the camera (e.g. the
/// parallax starfield) should run `.after(CameraFollow)` to avoid a frame of lag.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CameraFollow;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (
                reset_zoom.run_if(any_with_component::<CameraTarget>),
                zoom_camera.run_if(any_with_component::<CameraTarget>),
                follow_target
                    .in_set(CameraFollow)
                    .run_if(any_with_component::<CameraTarget>),
            ),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Sample4));
}

fn reset_zoom(
    target: Query<(), Added<CameraTarget>>,
    config: Res<GameConfig>,
    mut camera: Query<&mut Projection, With<Camera2d>>,
) {
    if target.is_empty() {
        return;
    }
    let Ok(mut projection) = camera.single_mut() else {
        return;
    };
    if let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scale = config.camera.default_scale;
    }
}

fn zoom_camera(
    scroll: Res<AccumulatedMouseScroll>,
    config: Res<GameConfig>,
    mut camera: Query<&mut Projection, With<Camera2d>>,
) {
    if scroll.delta.y == 0.0 {
        return;
    }
    let Ok(mut projection) = camera.single_mut() else {
        return;
    };
    if let Projection::Orthographic(ortho) = &mut *projection {
        let cam = &config.camera;
        let factor = (1.0 - scroll.delta.y * cam.zoom_sensitivity).clamp(0.5, 1.5);
        ortho.scale = (ortho.scale * factor).clamp(cam.min_scale, cam.max_scale);
    }
}

fn follow_target(
    target: Query<&Transform, (With<CameraTarget>, Without<Camera2d>)>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(target_transform) = target.single() else {
        return;
    };
    let Ok(mut cam_transform) = camera.single_mut() else {
        return;
    };
    cam_transform.translation.x = target_transform.translation.x;
    cam_transform.translation.y = target_transform.translation.y;
}
