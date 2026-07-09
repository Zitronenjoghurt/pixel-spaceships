use super::flames::{flame_bundle, flame_texture};
use super::flight::Flight;
use crate::plugins::{ActiveShip, CameraTarget, GameConfig};
use crate::state::AppState;
use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use pixel_spaceships_core::config::Modules;
use pixel_spaceships_core::ship::Ship;

#[derive(Resource)]
pub(super) struct WorldShip {
    pub(super) entity: Entity,
    image: Handle<Image>,
    flame_image: Handle<Image>,
}

pub(super) fn spawn_world_ship(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<GameConfig>,
    ship: Res<ActiveShip>,
) {
    let Some((image, size)) = rasterize_to_image(&ship.0, &config.modules, &mut images) else {
        return;
    };

    let (origin, _) = ship
        .0
        .bounds()
        .expect("bounds exist when rasterize succeeded");
    let center = origin.as_vec2() + size.as_vec2() / 2.0;

    let flame_image = flame_texture(&mut images, config.thrusters.flame_color.to_rgba());

    let entity = commands
        .spawn((
            Name::new("WorldShip"),
            Sprite {
                image: image.clone(),
                custom_size: Some(size.as_vec2()),
                ..default()
            },
            Transform::default(),
            Flight::default(),
            CameraTarget,
            DespawnOnExit(AppState::World),
        ))
        .with_children(|parent| {
            for (i, port) in ship.0.stats.thrust_profile.ports.iter().enumerate() {
                parent.spawn(flame_bundle(
                    i,
                    port,
                    center,
                    &config.thrusters,
                    &flame_image,
                ));
            }
        })
        .id();

    commands.insert_resource(WorldShip {
        entity,
        image,
        flame_image,
    });
}

pub(super) fn sync_world_ship(
    ship: Res<ActiveShip>,
    config: Res<GameConfig>,
    mut world_ship: ResMut<WorldShip>,
    mut sprites: Query<&mut Sprite>,
    mut images: ResMut<Assets<Image>>,
) {
    if !ship.is_changed() {
        return;
    }
    let Some((image, size)) = rasterize_to_image(&ship.0, &config.modules, &mut images) else {
        return;
    };
    if let Ok(mut sprite) = sprites.get_mut(world_ship.entity) {
        sprite.image = image.clone();
        sprite.custom_size = Some(size.as_vec2());
    }
    let stale = std::mem::replace(&mut world_ship.image, image);
    images.remove(&stale);
}

pub(super) fn despawn_world_ship(
    mut commands: Commands,
    world_ship: Option<Res<WorldShip>>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Some(world_ship) = world_ship {
        images.remove(&world_ship.image);
        images.remove(&world_ship.flame_image);
        commands.remove_resource::<WorldShip>();
    }
}

fn rasterize_to_image(
    ship: &Ship,
    modules: &Modules,
    images: &mut Assets<Image>,
) -> Option<(Handle<Image>, UVec2)> {
    let (origin, size) = ship.bounds()?;
    let mut image = Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image.data = Some(ship.rasterize(origin, size, modules).pixels);
    Some((images.add(image), size))
}
