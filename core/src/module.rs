use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ModuleKind {
    #[default]
    Hull,
    Thruster,
    Reactor,
}

impl ModuleKind {
    pub const ALL: [ModuleKind; 3] = [ModuleKind::Hull, ModuleKind::Thruster, ModuleKind::Reactor];

    pub fn def(self) -> ModuleDef {
        match self {
            ModuleKind::Hull => ModuleDef {
                kind: self,
                name: "Hull",
                mass: 1.0,
                ..Default::default()
            },
            ModuleKind::Thruster => ModuleDef {
                kind: self,
                name: "Thruster",
                mass: 2.0,
                power: -5.0,
                thrust: 10.0,
            },
            ModuleKind::Reactor => ModuleDef {
                kind: self,
                name: "Reactor",
                mass: 4.0,
                power: 20.0,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModuleDef {
    pub kind: ModuleKind,
    pub name: &'static str,
    pub mass: f32,
    pub power: f32,
    pub thrust: f32,
}
