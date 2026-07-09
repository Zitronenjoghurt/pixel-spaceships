use crate::config::Modules;
use crate::direction::Direction;
use crate::ship::cell::ShipCell;
use crate::ship::module::ShipModuleKind;
use crate::ship::raster::Raster;
use crate::ship::thrust::ThrustPort;
use glam::{IVec2, UVec2, Vec2};
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

    pub fn total_mass(&self, modules: &Modules) -> f32 {
        self.0.values().map(|c| modules.def(c.kind).mass).sum()
    }

    pub fn power_balance(&self, modules: &Modules) -> f32 {
        self.0.values().map(|c| modules.def(c.kind).power).sum()
    }

    pub fn center_of_mass(&self, modules: &Modules) -> Vec2 {
        let mut weighted = Vec2::ZERO;
        let mut total = 0.0;

        for (pos, cell) in &self.0 {
            let mass = modules.def(cell.kind).mass;
            weighted += (pos.as_vec2() + Vec2::splat(0.5)) * mass;
            total += mass;
        }

        if total > 0.0 {
            weighted / total
        } else {
            Vec2::ZERO
        }
    }

    pub fn center_of_thrust(&self, modules: &Modules) -> Option<Vec2> {
        let mut weighted = Vec2::ZERO;
        let mut total = 0.0;

        for (pos, cell) in &self.0 {
            let thrust = modules.def(cell.kind).thrust;
            if thrust <= 0.0 {
                continue;
            }
            weighted += (pos.as_vec2() + Vec2::splat(0.5)) * thrust;
            total += thrust;
        }

        (total > 0.0).then(|| weighted / total)
    }

    pub fn moment_of_inertia(&self, center_of_mass: Vec2, modules: &Modules) -> f32 {
        self.0
            .iter()
            .map(|(pos, cell)| {
                let mass = modules.def(cell.kind).mass;
                let arm = (pos.as_vec2() + Vec2::splat(0.5)) - center_of_mass;
                mass * arm.length_squared()
            })
            .sum()
    }

    pub fn rasterize(&self, origin: IVec2, size: UVec2, modules: &Modules) -> Raster {
        let mut raster = Raster::new(origin, size);
        for (pos, cell) in &self.0 {
            let local = *pos - origin;
            if local.x < 0 || local.y < 0 || local.x as u32 >= size.x || local.y as u32 >= size.y {
                continue;
            }
            let row = size.y - 1 - local.y as u32;
            let idx = ((row * size.x + local.x as u32) * 4) as usize;
            raster.pixels[idx..idx + 4].copy_from_slice(&modules.def(cell.kind).color.to_rgba());
        }
        raster
    }

    pub fn bounds(&self) -> Option<(IVec2, UVec2)> {
        let mut keys = self.0.keys();
        let first = *keys.next()?;
        let (mut min, mut max) = (first, first);
        for pos in keys {
            min = min.min(*pos);
            max = max.max(*pos);
        }
        Some((min, (max - min + IVec2::ONE).as_uvec2()))
    }

    pub fn thruster_ports(&self, center_of_mass: Vec2, modules: &Modules) -> Vec<ThrustPort> {
        let mut cells: Vec<(&IVec2, &ShipCell)> = self.0.iter().collect();
        cells.sort_unstable_by_key(|(pos, _)| (pos.x, pos.y));

        let mut ports = Vec::new();
        for (pos, cell) in cells {
            let thrust = modules.def(cell.kind).thrust;
            if thrust <= 0.0 {
                continue;
            }

            let arm = (pos.as_vec2() + Vec2::splat(0.5)) - center_of_mass;
            for push in Direction::ALL {
                if self.get(*pos + push.opposite().as_ivec2()).is_none() {
                    let dir = push.as_vec2();
                    ports.push(ThrustPort {
                        cell: *pos,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_writes_module_color() {
        let modules = Modules::default();
        let mut grid = ShipGrid::default();
        grid.place(IVec2::ZERO, ShipModuleKind::Reactor);
        let raster = grid.rasterize(IVec2::ZERO, UVec2::new(1, 1), &modules);
        assert_eq!(
            raster.pixels,
            modules.def(ShipModuleKind::Reactor).color.to_rgba()
        );
    }

    #[test]
    fn rasterize_flips_y_so_higher_cells_land_in_higher_rows() {
        let modules = Modules::default();
        let mut grid = ShipGrid::default();
        grid.place(IVec2::new(0, 1), ShipModuleKind::Hull); // top cell
        let raster = grid.rasterize(IVec2::ZERO, UVec2::new(1, 2), &modules);
        let hull = modules.def(ShipModuleKind::Hull).color.to_rgba();
        assert_eq!(raster.pixels[0..4].to_vec(), hull.to_vec()); // top row filled
        assert_eq!(raster.pixels[4..8].to_vec(), vec![0, 0, 0, 0]); // bottom row empty
    }

    #[test]
    fn rasterize_ignores_cells_outside_the_region() {
        let modules = Modules::default();
        let mut grid = ShipGrid::default();
        grid.place(IVec2::new(9, 9), ShipModuleKind::Hull);
        let raster = grid.rasterize(IVec2::ZERO, UVec2::new(2, 2), &modules);
        assert!(raster.pixels.iter().all(|&b| b == 0));
    }

    #[test]
    fn bounds_covers_every_cell() {
        let mut grid = ShipGrid::default();
        grid.place(IVec2::new(-1, 2), ShipModuleKind::Hull);
        grid.place(IVec2::new(3, -4), ShipModuleKind::Hull);
        assert_eq!(grid.bounds(), Some((IVec2::new(-1, -4), UVec2::new(5, 7))));
    }

    #[test]
    fn bounds_is_none_when_empty() {
        assert_eq!(ShipGrid::default().bounds(), None);
    }
}
