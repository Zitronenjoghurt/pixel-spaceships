use crate::config::Modules;
use crate::ship::grid::ShipGrid;
use crate::ship::thrust::ShipThrust;

#[derive(Debug, Default)]
pub struct ShipStats {
    pub center_of_mass: glam::Vec2,
    /// How much the ship resists rotation.
    pub moment_of_inertia: f32,
    pub power_balance: f32,
    pub thrust: ShipThrust,
    pub total_mass: f32,
}

impl ShipStats {
    pub fn compute(grid: &ShipGrid, modules: &Modules) -> Self {
        let center_of_mass = grid.center_of_mass(modules);
        let thrust_ports = grid.thruster_ports(center_of_mass, modules);
        Self {
            center_of_mass,
            moment_of_inertia: grid.moment_of_inertia(center_of_mass, modules),
            power_balance: grid.power_balance(modules),
            thrust: ShipThrust::from_ports(&thrust_ports),
            total_mass: grid.total_mass(modules),
        }
    }
}
