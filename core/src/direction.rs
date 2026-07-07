use glam::{IVec2, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    East,
    West,
    North,
    South,
}

impl Direction {
    pub const ALL: [Direction; 4] = [Self::East, Self::West, Self::North, Self::South];

    pub const fn as_ivec2(self) -> IVec2 {
        match self {
            Self::East => IVec2::new(1, 0),
            Self::West => IVec2::new(-1, 0),
            Self::North => IVec2::new(0, 1),
            Self::South => IVec2::new(0, -1),
        }
    }

    pub fn as_vec2(self) -> Vec2 {
        self.as_ivec2().as_vec2()
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::East => Self::West,
            Self::West => Self::East,
            Self::North => Self::South,
            Self::South => Self::North,
        }
    }
}
