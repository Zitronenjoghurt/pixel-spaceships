use super::flight::Flight;
use super::ship::WorldShip;
use crate::plugins::GameConfig;
use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use pixel_spaceships_core::config::Thrusters;
use pixel_spaceships_core::ship::thrust::ThrustPort;

/// An exhaust flame for a single thruster port, a child of the ship sprite.
#[derive(Component)]
pub(super) struct Flame {
    port: usize,
    /// Local position of the exhaust root (the cell edge), in sprite units.
    edge: Vec2,
    /// Outward exhaust direction, ship-local.
    dir: Vec2,
    /// Smoothed throttle driving the plume: snaps up when firing, eases down when
    /// released so the flame trails off instead of blinking out.
    lit: f32,
}

pub(super) fn flame_bundle(
    port_index: usize,
    port: &ThrustPort,
    center: Vec2,
    cfg: &Thrusters,
    image: &Handle<Image>,
) -> impl Bundle {
    let cell_center = port.cell.as_vec2() + Vec2::splat(0.5);
    let dir = port.push.opposite().as_vec2(); // exhaust points opposite thrust
    let edge = cell_center - center + dir * 0.5;
    let angle = (-dir.x).atan2(dir.y); // align local +Y with `dir`
    (
        Sprite {
            image: image.clone(),
            custom_size: Some(Vec2::new(cfg.flame_width, 0.0)),
            ..default()
        },
        Transform::from_translation(edge.extend(-0.5)).with_rotation(Quat::from_rotation_z(angle)),
        Visibility::Hidden,
        Flame {
            port: port_index,
            edge,
            dir,
            lit: 0.0,
        },
    )
}

pub(super) fn update_flames(
    time: Res<Time>,
    config: Res<GameConfig>,
    world_ship: Res<WorldShip>,
    flights: Query<&Flight>,
    mut flames: Query<(&mut Flame, &mut Sprite, &mut Transform, &mut Visibility)>,
) {
    let Ok(flight) = flights.get(world_ship.entity) else {
        return;
    };
    let cfg = &config.thrusters;
    let now = time.elapsed_secs();
    let decay = (-time.delta_secs() / cfg.spool_secs.max(1e-3)).exp();
    for (mut flame, mut sprite, mut transform, mut visibility) in &mut flames {
        let target = flight.throttles.get(flame.port).copied().unwrap_or(0.0);
        flame.lit = if target >= flame.lit {
            target
        } else {
            target + (flame.lit - target) * decay
        };
        if flame.lit < cfg.min_throttle {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;

        let phase = flame.port as f32;
        let turb = 0.6 * (now * 13.0 + phase * 1.7).sin()
            + 0.3 * (now * 27.0 + phase * 4.1).sin()
            + 0.1 * (now * 43.0 + phase * 2.3).sin();
        let length = (cfg.flame_length * flame.lit * (1.0 + cfg.flicker * turb)).max(0.0);
        sprite.custom_size = Some(Vec2::new(cfg.flame_width, length));

        let pos = flame.edge + flame.dir * (length * 0.5);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

pub(super) fn flame_texture(images: &mut Assets<Image>, color: [u8; 4]) -> Handle<Image> {
    const W: usize = 5;
    const H: usize = 12;
    let mid = [color[0] as f32, color[1] as f32, color[2] as f32];
    let core = [255.0, 250.0, 235.0];
    let cool = [225.0, 90.0, 35.0];
    let quant = |v: f32, steps: f32| (v * steps).round() / steps;
    let half_w = W as f32 / 2.0;

    let mut data = vec![0u8; W * H * 4];
    for y in 0..H {
        let from_root = (H - 1 - y) as f32 / (H as f32 - 1.0);
        let along = 1.0 - from_root;
        let half = 0.5 + along * (half_w - 0.5);
        let len_alpha = quant(0.3 + 0.7 * along, 4.0);
        for x in 0..W {
            let dx = (x as f32 + 0.5) - half_w;
            if dx.abs() > half {
                continue;
            }
            let across = 1.0 - (dx.abs() / half).min(1.0);
            let heat = quant(along * (0.55 + 0.45 * across), 5.0);
            let (r, g, b) = if heat >= 0.5 {
                let t = (heat - 0.5) / 0.5;
                (
                    lerp8(mid[0], core[0], t),
                    lerp8(mid[1], core[1], t),
                    lerp8(mid[2], core[2], t),
                )
            } else {
                let t = heat / 0.5;
                (
                    lerp8(cool[0], mid[0], t),
                    lerp8(cool[1], mid[1], t),
                    lerp8(cool[2], mid[2], t),
                )
            };

            let alpha = len_alpha * quant(0.35 + 0.65 * across, 3.0);
            let idx = (y * W + x) * 4;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = (alpha * 255.0) as u8;
        }
    }

    let mut image = Image::new_fill(
        Extent3d {
            width: W as u32,
            height: H as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image.data = Some(data);
    images.add(image)
}

fn lerp8(a: f32, b: f32, t: f32) -> u8 {
    (a + (b - a) * t) as u8
}
