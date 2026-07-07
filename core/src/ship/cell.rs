use crate::ship::module::ShipModuleKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShipCell {
    pub kind: ShipModuleKind,
}
