use crate::plugins::{ActiveShip, GameConfig};
use bevy::prelude::*;
use pixel_spaceships_core::ship::flight::{FlightInput, FlightState};

#[derive(Component, Default)]
pub(super) struct Flight {
    pub(super) state: FlightState,
    pub(super) throttles: Vec<f32>,
}

pub(super) fn fly_ship(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<GameConfig>,
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
        let flight = &mut *flight;
        flight.state.step(
            &ship.0.stats,
            &config.flight,
            input,
            dt,
            &mut flight.throttles,
        );
        transform.translation.x = flight.state.position.x;
        transform.translation.y = flight.state.position.y;
        transform.rotation = Quat::from_rotation_z(flight.state.rotation);
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
