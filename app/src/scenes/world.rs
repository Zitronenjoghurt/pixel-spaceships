use crate::plugins::{ActiveShip, CameraTarget, GameConfig};
use crate::state::AppState;
use crate::widgets::{TitleTag, viewport_root};
use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use egui_phosphor::regular;
use pixel_spaceships_core::config::Modules;
use pixel_spaceships_core::ship::Ship;
use pixel_spaceships_core::ship::flight::{FlightInput, FlightState};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::World), spawn_world_ship)
            .add_systems(OnExit(AppState::World), despawn_world_ship)
            .add_systems(
                FixedUpdate,
                fly_ship
                    .run_if(in_state(AppState::World))
                    .run_if(resource_exists::<WorldShip>),
            )
            .add_systems(
                Update,
                sync_world_ship
                    .run_if(in_state(AppState::World))
                    .run_if(resource_exists::<WorldShip>),
            )
            .add_systems(
                EguiPrimaryContextPass,
                world_hud.run_if(in_state(AppState::World)),
            );
    }
}

#[derive(Resource)]
struct WorldShip {
    entity: Entity,
    image: Handle<Image>,
}

#[derive(Component, Default)]
struct Flight(FlightState);

fn spawn_world_ship(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<GameConfig>,
    ship: Res<ActiveShip>,
) {
    let Some((image, size)) = rasterize_to_image(&ship.0, &config.modules, &mut images) else {
        return;
    };
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
        .id();
    commands.insert_resource(WorldShip { entity, image });
}

fn fly_ship(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    ship: Res<ActiveShip>,
    mut query: Query<(&mut Flight, &mut Transform)>,
) {
    let input = FlightInput {
        thrust: axis(&keys, KeyCode::KeyW, KeyCode::KeyS),
        strafe: axis(&keys, KeyCode::KeyD, KeyCode::KeyA),
        turn: axis(&keys, KeyCode::KeyQ, KeyCode::KeyE),
        brake: if keys.pressed(KeyCode::KeyC) {
            1.0
        } else {
            0.0
        },
    };
    let dt = time.delta_secs();
    for (mut flight, mut transform) in &mut query {
        flight.0.step(&ship.0.stats, input, dt);
        transform.translation.x = flight.0.position.x;
        transform.translation.y = flight.0.position.y;
        transform.rotation = Quat::from_rotation_z(flight.0.rotation);
    }
}

fn axis(keys: &ButtonInput<KeyCode>, pos: KeyCode, neg: KeyCode) -> f32 {
    let mut value = 0.0;
    if keys.pressed(pos) {
        value += 1.0;
    }
    if keys.pressed(neg) {
        value -= 1.0;
    }
    value
}

fn sync_world_ship(
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

fn despawn_world_ship(
    mut commands: Commands,
    world_ship: Option<Res<WorldShip>>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Some(world_ship) = world_ship {
        images.remove(&world_ship.image);
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

fn world_hud(mut contexts: EguiContexts, mut next: ResMut<NextState<AppState>>) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mut root = viewport_root(ctx, "world");

    egui::Panel::top("world_top").show(&mut root, |ui| {
        ui.horizontal(|ui| {
            ui.add(TitleTag);
            ui.separator();
            if ui.button(format!("{} Editor", regular::WRENCH)).clicked() {
                next.set(AppState::ShipEditor);
            }
        });
    });
    Ok(())
}
