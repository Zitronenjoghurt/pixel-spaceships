use crate::direction::Direction;
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThrustPort {
    /// Cardinal direction the ship is pushed.
    pub push: Direction,
    /// Linear contribution at full throttle (`push * thrust`).
    pub force: Vec2,
    /// Angular contribution at full throttle, positive is counterclockwise.
    pub torque: f32,
}

/// Peak thrust available in each cardinal direction and angle, at full throttle.
/// These are upper bounds that ignore parasitic spin: firing every east port at
/// once also rotates the ship unless those ports are balanced around the CoM.
#[derive(Debug, Default, Clone, Copy)]
pub struct ShipThrust {
    pub east: f32,
    pub west: f32,
    pub north: f32,
    pub south: f32,
    pub cw: f32,
    pub ccw: f32,
}

impl ShipThrust {
    pub fn from_ports(ports: &[ThrustPort]) -> Self {
        let mut thrust = Self::default();
        for port in ports {
            match port.push {
                Direction::East => thrust.east += port.force.x,
                Direction::West => thrust.west -= port.force.x,
                Direction::North => thrust.north += port.force.y,
                Direction::South => thrust.south -= port.force.y,
            }
            if port.torque > 0.0 {
                thrust.ccw += port.torque;
            } else if port.torque < 0.0 {
                thrust.cw -= port.torque;
            }
        }
        thrust
    }
}
