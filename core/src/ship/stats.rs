use crate::config::Modules;
use crate::ship::grid::ShipGrid;
use crate::ship::thrust::{ShipThrust, ThrustProfile};

#[derive(Debug, Default)]
pub struct ShipStats {
    pub center_of_mass: glam::Vec2,
    pub center_of_thrust: Option<glam::Vec2>,
    pub moment_of_inertia: f32,
    pub power_balance: f32,
    pub thrust: ShipThrust,
    pub total_mass: f32,
    pub thrust_profile: ThrustProfile,
}

impl ShipStats {
    pub fn compute(grid: &ShipGrid, modules: &Modules) -> Self {
        let center_of_mass = grid.center_of_mass(modules);
        let ports = grid.thruster_ports(center_of_mass, modules);
        let thrust_profile = ThrustProfile::solve(ports);
        Self {
            center_of_mass,
            center_of_thrust: grid.center_of_thrust(modules),
            moment_of_inertia: grid.moment_of_inertia(center_of_mass, modules),
            power_balance: grid.power_balance(modules),
            thrust: thrust_profile.capability(),
            total_mass: grid.total_mass(modules),
            thrust_profile,
        }
    }
}
