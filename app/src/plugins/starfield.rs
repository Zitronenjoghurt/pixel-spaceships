use crate::plugins::{CameraFollow, GameConfig};
use crate::state::AppState;
use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub struct StarfieldPlugin;

impl Plugin for StarfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_star_texture)
            .add_systems(OnEnter(AppState::World), spawn_stars)
            .add_systems(
                Update,
                parallax_stars
                    .after(CameraFollow)
                    .run_if(in_state(AppState::World)),
            );
    }
}

#[derive(Resource)]
struct StarTexture(Handle<Image>);

#[derive(Component)]
struct Star {
    base: Vec2,
    parallax: f32,
}

fn load_star_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    commands.insert_resource(StarTexture(images.add(image)));
}

fn spawn_stars(
    mut commands: Commands,
    mut clear: ResMut<ClearColor>,
    config: Res<GameConfig>,
    texture: Res<StarTexture>,
) {
    clear.0 = Color::BLACK;

    let tile = config.starfield.tile;
    let mut seed = 1u32;
    for layer in &config.starfield.layers {
        let [r, g, b, _] = layer.color.to_rgba();
        for _ in 0..layer.count {
            let x = hash01(seed) * tile;
            let y = hash01(seed ^ 0x9e37_79b9) * tile;
            let size = lerp(
                layer.min_size,
                layer.max_size,
                hash01(seed.wrapping_mul(2_654_435_761)),
            );
            let alpha = lerp(
                layer.min_alpha,
                layer.max_alpha,
                hash01(seed.wrapping_mul(40_503) ^ 0x68e3_1da4),
            );
            seed = seed.wrapping_add(1);

            commands.spawn((
                Sprite {
                    image: texture.0.clone(),
                    color: Color::srgb_u8(r, g, b).with_alpha(alpha),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, layer.z),
                Star {
                    base: Vec2::new(x, y),
                    parallax: layer.parallax,
                },
                DespawnOnExit(AppState::World),
            ));
        }
    }
}

fn parallax_stars(
    config: Res<GameConfig>,
    camera: Query<&Transform, (With<Camera2d>, Without<Star>)>,
    mut stars: Query<(&Star, &mut Transform)>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let tile = config.starfield.tile;
    let c = camera.translation.truncate();
    let half = tile * 0.5;
    for (star, mut transform) in &mut stars {
        let drift = c * (1.0 - star.parallax);
        let x = (star.base.x - drift.x).rem_euclid(tile);
        let y = (star.base.y - drift.y).rem_euclid(tile);
        transform.translation.x = c.x + x - half;
        transform.translation.y = c.y + y - half;
    }
}

fn hash01(mut x: u32) -> f32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}

fn lerp(min: f32, max: f32, t: f32) -> f32 {
    min + (max - min) * t
}
