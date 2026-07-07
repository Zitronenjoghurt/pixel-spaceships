use glam::IVec2;
use serde::{Deserialize, Serialize};

pub mod cell;
mod grid;
pub mod module;
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

    pub fn place(&mut self, at: IVec2, kind: module::ShipModuleKind) {
        self.grid.place(at, kind);
        self.recompute_stats();
    }

    pub fn remove(&mut self, at: IVec2) -> Option<cell::ShipCell> {
        self.grid.remove(at)
    }

    fn recompute_stats(&mut self) {
        self.stats = stats::ShipStats::compute(&self.grid);
    }
}
