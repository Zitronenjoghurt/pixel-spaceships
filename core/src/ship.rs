use crate::ModuleKind;
use glam::IVec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub kind: ModuleKind,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ship {
    cells: HashMap<IVec2, Cell>,
}

impl Ship {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn place(&mut self, at: IVec2, kind: ModuleKind) {
        self.cells.insert(at, Cell { kind });
    }

    pub fn remove(&mut self, at: IVec2) -> Option<Cell> {
        self.cells.remove(&at)
    }

    pub fn get(&self, at: IVec2) -> Option<&Cell> {
        self.cells.get(&at)
    }

    pub fn cells(&self) -> impl Iterator<Item = (&IVec2, &Cell)> {
        self.cells.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn total_mass(&self) -> f32 {
        self.cells.values().map(|c| c.kind.def().mass).sum()
    }

    pub fn power_balance(&self) -> f32 {
        self.cells.values().map(|c| c.kind.def().power).sum()
    }

    pub fn total_thrust(&self) -> f32 {
        self.cells.values().map(|c| c.kind.def().thrust).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_and_thrust_add_up() {
        let mut ship = Ship::new();
        ship.place(IVec2::new(0, 0), ModuleKind::Reactor);
        ship.place(IVec2::new(1, 0), ModuleKind::Thruster);

        assert_eq!(ship.power_balance(), 15.0);
        assert_eq!(ship.total_thrust(), 10.0);
        assert_eq!(ship.total_mass(), 6.0);
    }

    #[test]
    fn remove_clears_cell() {
        let mut ship = Ship::new();
        ship.place(IVec2::new(0, 0), ModuleKind::Hull);
        assert!(ship.get(IVec2::new(0, 0)).is_some());
        ship.remove(IVec2::new(0, 0));
        assert!(ship.is_empty());
    }
}
