use crate::direction::Direction;
use crate::ship::cell::ShipCell;
use crate::ship::module::ShipModuleKind;
use crate::ship::thrust::ThrustPort;
use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ShipGrid(HashMap<IVec2, ShipCell>);

impl ShipGrid {
    pub fn place(&mut self, at: IVec2, kind: ShipModuleKind) {
        self.0.insert(at, ShipCell { kind });
    }

    pub fn remove(&mut self, at: IVec2) -> Option<ShipCell> {
        self.0.remove(&at)
    }

    pub fn get(&self, at: IVec2) -> Option<&ShipCell> {
        self.0.get(&at)
    }

    pub fn cells(&self) -> impl Iterator<Item = (&IVec2, &ShipCell)> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn total_mass(&self) -> f32 {
        self.0.values().map(|c| c.kind.def().mass).sum()
    }

    pub fn power_balance(&self) -> f32 {
        self.0.values().map(|c| c.kind.def().power).sum()
    }

    pub fn center_of_mass(&self) -> Vec2 {
        let mut weighted = Vec2::ZERO;
        let mut total = 0.0;

        for (pos, cell) in &self.0 {
            let mass = cell.kind.def().mass;
            weighted += (pos.as_vec2() + Vec2::splat(0.5)) * mass;
            total += mass;
        }

        if total > 0.0 {
            weighted / total
        } else {
            Vec2::ZERO
        }
    }

    pub fn moment_of_inertia(&self, center_of_mass: Vec2) -> f32 {
        self.0
            .iter()
            .map(|(pos, cell)| {
                let mass = cell.kind.def().mass;
                let arm = (pos.as_vec2() + Vec2::splat(0.5)) - center_of_mass;
                mass * arm.length_squared()
            })
            .sum()
    }

    pub fn thruster_ports(&self, center_of_mass: Vec2) -> Vec<ThrustPort> {
        let mut ports = Vec::new();
        for (pos, cell) in &self.0 {
            let thrust = cell.kind.def().thrust;
            if thrust <= 0.0 {
                continue;
            }

            let arm = (pos.as_vec2() + Vec2::splat(0.5)) - center_of_mass;
            for push in Direction::ALL {
                if self.get(*pos + push.opposite().as_ivec2()).is_none() {
                    let dir = push.as_vec2();
                    ports.push(ThrustPort {
                        push,
                        force: dir * thrust,
                        torque: arm.perp_dot(dir) * thrust,
                    });
                }
            }
        }
        ports
    }
}
