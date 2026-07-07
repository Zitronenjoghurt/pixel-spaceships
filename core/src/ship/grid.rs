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

    /// Rasterizes the cells within `[origin, origin + size)` into an RGBA8 [`Raster`].
    /// Cells outside the region are ignored; empty cells stay transparent.
    pub fn rasterize(&self, origin: IVec2, size: UVec2) -> Raster {
        let mut raster = Raster::new(origin, size);
        for (pos, cell) in &self.0 {
            let local = *pos - origin;
            if local.x < 0 || local.y < 0 || local.x as u32 >= size.x || local.y as u32 >= size.y {
                continue;
            }
            let row = size.y - 1 - local.y as u32;
            let idx = ((row * size.x + local.x as u32) * 4) as usize;
            raster.pixels[idx..idx + 4].copy_from_slice(&cell.kind.def().color.to_rgba());
        }
        raster
    }

    /// The tight `(origin, size)` covering every cell, or `None` when the grid is empty.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_writes_module_color() {
        let mut grid = ShipGrid::default();
        grid.place(IVec2::ZERO, ShipModuleKind::Reactor);
        let raster = grid.rasterize(IVec2::ZERO, UVec2::new(1, 1));
        assert_eq!(raster.pixels, ShipModuleKind::Reactor.def().color.to_rgba());
    }

    #[test]
    fn rasterize_flips_y_so_higher_cells_land_in_higher_rows() {
        let mut grid = ShipGrid::default();
        grid.place(IVec2::new(0, 1), ShipModuleKind::Hull); // top cell
        let raster = grid.rasterize(IVec2::ZERO, UVec2::new(1, 2));
        let hull = ShipModuleKind::Hull.def().color.to_rgba();
        assert_eq!(raster.pixels[0..4].to_vec(), hull.to_vec()); // top row filled
        assert_eq!(raster.pixels[4..8].to_vec(), vec![0, 0, 0, 0]); // bottom row empty
    }

    #[test]
    fn rasterize_ignores_cells_outside_the_region() {
        let mut grid = ShipGrid::default();
        grid.place(IVec2::new(9, 9), ShipModuleKind::Hull);
        let raster = grid.rasterize(IVec2::ZERO, UVec2::new(2, 2));
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
