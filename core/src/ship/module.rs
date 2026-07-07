use crate::color::Color;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, EnumIter)]
pub enum ShipModuleKind {
    #[default]
    Hull,
    Thruster,
    Reactor,
}

// Module colors loosely follow this palette: https://lospec.com/palette-list/endesga-32
impl ShipModuleKind {
    pub fn def(self) -> ShipModuleDef {
        match self {
            ShipModuleKind::Hull => ShipModuleDef {
                kind: self,
                name: "Hull",
                color: Color::rgb_hex(0x8B9BB4),
                mass: 1.0,
                ..Default::default()
            },
            ShipModuleKind::Thruster => ShipModuleDef {
                kind: self,
                name: "Thruster",
                color: Color::rgb_hex(0xF77622),
                mass: 2.0,
                power: -5.0,
                thrust: 10.0,
            },
            ShipModuleKind::Reactor => ShipModuleDef {
                kind: self,
                name: "Reactor",
                color: Color::rgb_hex(0x3E8948),
                mass: 4.0,
                power: 20.0,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShipModuleDef {
    pub kind: ShipModuleKind,
    pub name: &'static str,
    pub color: Color,
    pub mass: f32,
    pub power: f32,
    pub thrust: f32,
}
