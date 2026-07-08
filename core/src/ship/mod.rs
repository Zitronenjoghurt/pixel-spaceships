use crate::config::Modules;
use crate::ship::raster::Raster;
use glam::{IVec2, UVec2};
use serde::{Deserialize, Serialize};

pub mod cell;
pub mod flight;
mod grid;
pub mod module;
pub mod raster;
pub mod stats;
pub mod thrust;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ship {
    grid: grid::ShipGrid,
    #[serde(default, skip)]
    pub stats: stats::ShipStats,
}

impl Ship {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn place(&mut self, at: IVec2, kind: module::ShipModuleKind, modules: &Modules) {
        self.grid.place(at, kind);
        self.recompute_stats(modules);
    }

    pub fn remove(&mut self, at: IVec2, modules: &Modules) -> Option<cell::ShipCell> {
        let removed = self.grid.remove(at);
        if removed.is_some() {
            self.recompute_stats(modules);
        }
        removed
    }

    pub fn rasterize(&self, origin: IVec2, size: UVec2, modules: &Modules) -> Raster {
        self.grid.rasterize(origin, size, modules)
    }

    pub fn bounds(&self) -> Option<(IVec2, UVec2)> {
        self.grid.bounds()
    }

    fn recompute_stats(&mut self, modules: &Modules) {
        self.stats = stats::ShipStats::compute(&self.grid, modules);
    }
}
