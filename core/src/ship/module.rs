use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, EnumIter)]
pub enum ShipModuleKind {
    #[default]
    Hull,
    Thruster,
    Reactor,
}

impl ShipModuleKind {
    pub fn name(self) -> &'static str {
        match self {
            ShipModuleKind::Hull => "Hull",
            ShipModuleKind::Thruster => "Thruster",
            ShipModuleKind::Reactor => "Reactor",
        }
    }
}
