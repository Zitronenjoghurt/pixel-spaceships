use crate::ship::stats::ShipStats;
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
    pub fn step(&mut self, stats: &ShipStats, input: FlightInput, dt: f32) {
        let thrust = input.thrust.clamp(-1.0, 1.0);
        let strafe = input.strafe.clamp(-1.0, 1.0);
        let turn = input.turn.clamp(-1.0, 1.0);

        let forward = if thrust >= 0.0 {
            stats.thrust.north
        } else {
            stats.thrust.south
        };
        let lateral = if strafe >= 0.0 {
            stats.thrust.east
        } else {
            stats.thrust.west
        };
        let spin = if turn >= 0.0 {
            stats.thrust.ccw
        } else {
            stats.thrust.cw
        };

        let local_force = Vec2::new(strafe * lateral, thrust * forward);
        let world_force = Vec2::from_angle(self.rotation).rotate(local_force);
        let torque = turn * spin;

        if stats.total_mass > 0.0 {
            self.velocity += world_force / stats.total_mass * dt;
        }
        if stats.moment_of_inertia > 0.0 {
            self.angular_velocity += torque / stats.moment_of_inertia * dt;
        }

        let brake = input.brake.clamp(0.0, 1.0);
        if brake > 0.0 {
            if stats.total_mass > 0.0 {
                let mut local_v = Vec2::from_angle(-self.rotation).rotate(self.velocity);
                let cap_x = if local_v.x > 0.0 {
                    stats.thrust.west
                } else {
                    stats.thrust.east
                };
                let cap_y = if local_v.y > 0.0 {
                    stats.thrust.south
                } else {
                    stats.thrust.north
                };
                local_v.x = brake_toward_zero(local_v.x, brake * cap_x / stats.total_mass * dt);
                local_v.y = brake_toward_zero(local_v.y, brake * cap_y / stats.total_mass * dt);
                self.velocity = Vec2::from_angle(self.rotation).rotate(local_v);
            }
            if stats.moment_of_inertia > 0.0 {
                let cap = if self.angular_velocity > 0.0 {
                    stats.thrust.cw
                } else {
                    stats.thrust.ccw
                };
                self.angular_velocity = brake_toward_zero(
                    self.angular_velocity,
                    brake * cap / stats.moment_of_inertia * dt,
                );
            }
        }

        self.position += self.velocity * dt;
        self.rotation += self.angular_velocity * dt;
    }
}

fn brake_toward_zero(value: f32, amount: f32) -> f32 {
    value.signum() * (value.abs() - amount).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Modules;
    use crate::ship::Ship;
    use crate::ship::module::ShipModuleKind;
    use crate::ship::thrust::ShipThrust;
    use glam::IVec2;

    #[test]
    fn forward_thrust_accelerates_along_local_up() {
        let mut ship = Ship::new();
        ship.place(IVec2::ZERO, ShipModuleKind::Thruster, &Modules::default());

        let mut flight = FlightState::default();
        let input = FlightInput {
            thrust: 1.0,
            ..Default::default()
        };
        flight.step(&ship.stats, input, 1.0);

        assert_eq!(flight.velocity, Vec2::new(0.0, 5.0));
        assert_eq!(flight.position, Vec2::new(0.0, 5.0));
    }

    #[test]
    fn thrust_follows_ship_rotation() {
        let mut ship = Ship::new();
        ship.place(IVec2::ZERO, ShipModuleKind::Thruster, &Modules::default());

        let mut flight = FlightState {
            rotation: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };
        flight.step(
            &ship.stats,
            FlightInput {
                thrust: 1.0,
                ..Default::default()
            },
            1.0,
        );

        assert!(flight.velocity.x < 0.0);
        assert!(flight.velocity.y.abs() < 1e-5);
    }

    #[test]
    fn brake_bleeds_off_momentum() {
        let stats = ShipStats {
            total_mass: 2.0,
            moment_of_inertia: 4.0,
            thrust: ShipThrust {
                east: 10.0,
                west: 10.0,
                north: 10.0,
                south: 10.0,
                cw: 6.0,
                ccw: 6.0,
            },
            ..Default::default()
        };
        let mut flight = FlightState {
            velocity: Vec2::new(10.0, 0.0),
            angular_velocity: 2.0,
            ..Default::default()
        };
        flight.step(
            &stats,
            FlightInput {
                brake: 1.0,
                ..Default::default()
            },
            0.1,
        );

        assert!(flight.velocity.x > 0.0 && flight.velocity.x < 10.0);
        assert!(flight.angular_velocity > 0.0 && flight.angular_velocity < 2.0);
    }

    #[test]
    fn braking_authority_scales_with_thrust() {
        let weak = ShipStats {
            total_mass: 100.0,
            thrust: ShipThrust {
                west: 5.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let strong = ShipStats {
            total_mass: 100.0,
            thrust: ShipThrust {
                west: 50.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let base = FlightState {
            velocity: Vec2::new(10.0, 0.0),
            ..Default::default()
        };
        let brake = FlightInput {
            brake: 1.0,
            ..Default::default()
        };

        let mut weak_flight = base;
        let mut strong_flight = base;
        weak_flight.step(&weak, brake, 0.1);
        strong_flight.step(&strong, brake, 0.1);

        assert!(strong_flight.velocity.x < weak_flight.velocity.x);
    }
}
