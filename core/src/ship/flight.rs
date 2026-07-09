use crate::config::Flight;
use crate::ship::stats::ShipStats;
use crate::ship::thrust::Command;
use glam::Vec2;

#[derive(Debug, Default, Clone, Copy)]
pub struct FlightInput {
    /// Forward (+) / reverse (-): fires the ship's north / south thrust.
    pub thrust: f32,
    /// Strafe right (+) / left (-): fires the ship's east / west thrust.
    pub strafe: f32,
    /// Turn counter-clockwise (+) / clockwise (-).
    pub turn: f32,
    /// Braking assist in `[0, 1]`: fires the thrusters opposing current motion.
    pub brake: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FlightState {
    pub position: Vec2,
    pub rotation: f32,
    pub velocity: Vec2,
    pub angular_velocity: f32,
}

impl FlightState {
    pub fn step(
        &mut self,
        stats: &ShipStats,
        tuning: &Flight,
        input: FlightInput,
        dt: f32,
        throttles: &mut Vec<f32>,
    ) {
        let mut commands = [0.0f32; 6];

        let thrust = input.thrust.clamp(-1.0, 1.0);
        let strafe = input.strafe.clamp(-1.0, 1.0);
        let turn = input.turn.clamp(-1.0, 1.0);
        if thrust >= 0.0 {
            commands[Command::Forward.index()] = thrust;
        } else {
            commands[Command::Back.index()] = -thrust;
        }
        if strafe >= 0.0 {
            commands[Command::Right.index()] = strafe;
        } else {
            commands[Command::Left.index()] = -strafe;
        }
        if turn >= 0.0 {
            commands[Command::Ccw.index()] = turn;
        } else {
            commands[Command::Cw.index()] = -turn;
        }

        let eff_mass = effective(stats.total_mass, tuning.mass_response);
        let eff_inertia = effective(stats.moment_of_inertia, tuning.mass_response);

        let brake = input.brake.clamp(0.0, 1.0);
        if brake > 0.0 {
            self.add_brake_commands(stats, eff_mass, eff_inertia, brake, dt, &mut commands);
        }
        for command in commands.iter_mut() {
            *command = command.clamp(0.0, 1.0);
        }

        let profile = &stats.thrust_profile;
        profile.resolve_into(&commands, throttles);
        let (local_force, torque) = profile.net(throttles);
        let world_force = Vec2::from_angle(self.rotation).rotate(local_force);

        if eff_mass > 0.0 {
            self.velocity += world_force / eff_mass * dt;
        }
        if eff_inertia > 0.0 {
            self.angular_velocity += torque / eff_inertia * dt;
        }

        self.apply_tuning(tuning, dt);

        self.position += self.velocity * dt;
        self.rotation += self.angular_velocity * dt;
    }

    fn apply_tuning(&mut self, tuning: &Flight, dt: f32) {
        if tuning.linear_damping > 0.0 {
            self.velocity *= (1.0 - tuning.linear_damping * dt).max(0.0);
        }
        if tuning.angular_damping > 0.0 {
            self.angular_velocity *= (1.0 - tuning.angular_damping * dt).max(0.0);
        }
        if tuning.max_speed > 0.0 {
            let speed = self.velocity.length();
            if speed > tuning.max_speed {
                self.velocity *= tuning.max_speed / speed;
            }
        }
        if tuning.max_angular_speed > 0.0 {
            self.angular_velocity = self
                .angular_velocity
                .clamp(-tuning.max_angular_speed, tuning.max_angular_speed);
        }
    }

    fn add_brake_commands(
        &self,
        stats: &ShipStats,
        mass: f32,
        inertia: f32,
        brake: f32,
        dt: f32,
        commands: &mut [f32; 6],
    ) {
        let cap = &stats.thrust;
        if mass > 0.0 {
            let local_v = Vec2::from_angle(-self.rotation).rotate(self.velocity);
            if local_v.y > 0.0 {
                commands[Command::Back.index()] +=
                    brake * brake_intensity(local_v.y, cap.south, mass, dt);
            } else if local_v.y < 0.0 {
                commands[Command::Forward.index()] +=
                    brake * brake_intensity(-local_v.y, cap.north, mass, dt);
            }
            if local_v.x > 0.0 {
                commands[Command::Left.index()] +=
                    brake * brake_intensity(local_v.x, cap.west, mass, dt);
            } else if local_v.x < 0.0 {
                commands[Command::Right.index()] +=
                    brake * brake_intensity(-local_v.x, cap.east, mass, dt);
            }
        }

        if inertia > 0.0 {
            if self.angular_velocity > 0.0 {
                commands[Command::Cw.index()] +=
                    brake * brake_intensity(self.angular_velocity, cap.cw, inertia, dt);
            } else if self.angular_velocity < 0.0 {
                commands[Command::Ccw.index()] +=
                    brake * brake_intensity(-self.angular_velocity, cap.ccw, inertia, dt);
            }
        }
    }
}

fn effective(value: f32, response: f32) -> f32 {
    if value > 0.0 {
        value.powf(response)
    } else {
        0.0
    }
}

fn brake_intensity(speed: f32, force: f32, inertia: f32, dt: f32) -> f32 {
    let accel = force / inertia;
    if accel <= 0.0 {
        return 0.0;
    }
    (speed / (accel * dt)).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Modules;
    use crate::ship::Ship;
    use crate::ship::module::ShipModuleKind::{Hull, Thruster};
    use glam::IVec2;

    fn balanced_ship() -> Ship {
        let modules = Modules::default();
        let mut ship = Ship::new();
        ship.place(IVec2::new(0, 0), Thruster, &modules);
        ship.place(IVec2::new(2, 0), Thruster, &modules);
        ship
    }

    const NO_TUNING: Flight = Flight {
        max_speed: 0.0,
        max_angular_speed: 0.0,
        linear_damping: 0.0,
        angular_damping: 0.0,
        mass_response: 1.0,
    };

    fn step(ship: &Ship, input: FlightInput, dt: f32) -> FlightState {
        let mut flight = FlightState::default();
        let mut throttles = Vec::new();
        flight.step(&ship.stats, &NO_TUNING, input, dt, &mut throttles);
        flight
    }

    #[test]
    fn balanced_forward_thrust_is_clean() {
        let flight = step(
            &balanced_ship(),
            FlightInput {
                thrust: 1.0,
                ..Default::default()
            },
            1.0,
        );
        assert!(flight.velocity.y > 0.0);
        assert!(flight.velocity.x.abs() < 1e-2);
        assert!(flight.angular_velocity.abs() < 1e-2);
    }

    #[test]
    fn single_off_center_thruster_cannot_translate_cleanly() {
        let modules = Modules::default();
        let mut ship = Ship::new();
        ship.place(IVec2::new(0, 0), Thruster, &modules);
        ship.place(
            IVec2::new(5, 0),
            crate::ship::module::ShipModuleKind::Hull,
            &modules,
        );

        let flight = step(
            &ship,
            FlightInput {
                thrust: 1.0,
                ..Default::default()
            },
            1.0,
        );
        assert!(flight.velocity.length() < 0.2);
    }

    #[test]
    fn strafe_with_fore_aft_pair_is_exactly_clean() {
        let modules = Modules::default();
        let mut ship = Ship::new();
        ship.place(IVec2::new(0, 3), Thruster, &modules);
        ship.place(IVec2::new(0, -3), Thruster, &modules);

        let flight = step(
            &ship,
            FlightInput {
                strafe: 1.0,
                ..Default::default()
            },
            1.0,
        );
        assert!(flight.velocity.x > 0.0);
        assert!(flight.velocity.y.abs() < 1e-6);
        assert!(flight.angular_velocity.abs() < 1e-6);
    }

    #[test]
    fn thrust_follows_ship_rotation() {
        let ship = balanced_ship();
        let mut flight = FlightState {
            rotation: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };
        let mut throttles = Vec::new();
        flight.step(
            &ship.stats,
            &NO_TUNING,
            FlightInput {
                thrust: 1.0,
                ..Default::default()
            },
            1.0,
            &mut throttles,
        );
        assert!(flight.velocity.x < 0.0);
        assert!(flight.velocity.y.abs() < 1e-2);
    }

    #[test]
    fn brake_slows_a_real_ship_without_reversing() {
        let ship = balanced_ship();
        let mut flight = FlightState {
            velocity: Vec2::new(0.0, 5.0),
            ..Default::default()
        };
        let mut throttles = Vec::new();
        for _ in 0..20 {
            flight.step(
                &ship.stats,
                &NO_TUNING,
                FlightInput {
                    brake: 1.0,
                    ..Default::default()
                },
                0.1,
                &mut throttles,
            );
        }
        assert!(flight.velocity.y >= 0.0 && flight.velocity.y < 5.0);
    }

    #[test]
    fn symmetric_ship_turns_evenly() {
        let modules = Modules::default();
        let mut ship = Ship::new();
        ship.place(IVec2::new(0, 0), Hull, &modules);
        for (x, y) in [(3, 0), (-3, 0), (0, 3), (0, -3)] {
            ship.place(IVec2::new(x, y), Thruster, &modules);
        }
        let t = &ship.stats.thrust;
        assert!(t.cw > 0.0);
        assert!(
            (t.cw - t.ccw).abs() <= t.cw * 1e-4,
            "cw {} ccw {}",
            t.cw,
            t.ccw
        );
    }

    #[test]
    fn stats_are_deterministic() {
        let modules = Modules::default();
        let build = || {
            let mut ship = Ship::new();
            for (x, y) in [(0, 0), (2, 0), (0, 3), (-2, -1), (1, -3)] {
                ship.place(IVec2::new(x, y), Thruster, &modules);
            }
            ship
        };
        let (a, b) = (build().stats.thrust, build().stats.thrust);
        assert_eq!(a.cw, b.cw);
        assert_eq!(a.ccw, b.ccw);
        assert_eq!(a.north, b.north);
        assert_eq!(a.east, b.east);
    }

    #[test]
    fn softer_mass_response_boosts_a_loaded_ship() {
        let modules = Modules::default();
        let mut ship = Ship::new();
        ship.place(IVec2::new(0, 0), Thruster, &modules);
        ship.place(IVec2::new(2, 0), Thruster, &modules);
        for (x, y) in [(1, 0), (0, 1), (1, 1), (2, 1)] {
            ship.place(IVec2::new(x, y), Hull, &modules);
        }
        assert!(ship.stats.total_mass > 1.0);

        let accel_with = |k: f32| {
            let mut flight = FlightState::default();
            let mut throttles = Vec::new();
            flight.step(
                &ship.stats,
                &Flight {
                    mass_response: k,
                    ..NO_TUNING
                },
                FlightInput {
                    thrust: 1.0,
                    ..Default::default()
                },
                0.1,
                &mut throttles,
            );
            flight.velocity.length()
        };
        assert!(accel_with(1.0) > 0.0);
        assert!(accel_with(0.8) > accel_with(1.0) * 1.05);
    }

    #[test]
    fn tuning_caps_top_speed() {
        let ship = balanced_ship();
        let tuning = Flight::default();
        let mut flight = FlightState::default();
        let mut throttles = Vec::new();
        for _ in 0..2000 {
            flight.step(
                &ship.stats,
                &tuning,
                FlightInput {
                    thrust: 1.0,
                    ..Default::default()
                },
                1.0 / 60.0,
                &mut throttles,
            );
        }
        assert!(
            flight.velocity.length() <= tuning.max_speed + 1e-3,
            "speed {} exceeded cap {}",
            flight.velocity.length(),
            tuning.max_speed,
        );
    }
}
