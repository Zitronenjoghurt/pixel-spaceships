use glam::{IVec2, UVec2};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    pub origin: IVec2,
    pub size: UVec2,
    pub pixels: Vec<u8>,
}

impl Raster {
    pub fn new(origin: IVec2, size: UVec2) -> Self {
        Self {
            origin,
            size,
            pixels: vec![0; (size.x * size.y * 4) as usize],
        }
    }
}
