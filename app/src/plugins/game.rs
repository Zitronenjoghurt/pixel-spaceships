use bevy::prelude::*;
use pixel_spaceships_core::config::Config;
use pixel_spaceships_core::ship::Ship;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .init_resource::<ActiveShip>();
    }
}

#[derive(Resource, Default)]
pub struct ActiveShip(pub Ship);

#[derive(Resource, Default, Deref)]
pub struct GameConfig(pub Config);
