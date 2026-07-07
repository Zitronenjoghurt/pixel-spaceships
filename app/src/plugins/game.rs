use bevy::prelude::*;
use pixel_spaceships_core::ship::Ship;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveShip>();
        // e.g. app.add_systems(FixedUpdate, step_flight.run_if(in_state(AppState::World)));
    }
}

#[derive(Resource, Default)]
pub struct ActiveShip(pub Ship);
