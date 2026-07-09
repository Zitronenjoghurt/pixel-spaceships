mod flames;
mod flight;
mod hud;
mod ship;

use crate::state::AppState;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use ship::WorldShip;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::World), ship::spawn_world_ship)
            .add_systems(OnExit(AppState::World), ship::despawn_world_ship)
            .add_systems(
                FixedUpdate,
                flight::fly_ship
                    .run_if(in_state(AppState::World))
                    .run_if(resource_exists::<WorldShip>),
            )
            .add_systems(
                Update,
                (ship::sync_world_ship, flames::update_flames)
                    .run_if(in_state(AppState::World))
                    .run_if(resource_exists::<WorldShip>),
            )
            .add_systems(
                EguiPrimaryContextPass,
                hud::world_hud.run_if(in_state(AppState::World)),
            );
    }
}
