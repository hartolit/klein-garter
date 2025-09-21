use crate::prelude::Glyph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Terrain {
    pub style: Glyph,
    pub z_index: u8,
    // TODO: pub is_collidable: bool - (leaving out for now)
}

impl Terrain {
    pub fn new(style: Glyph, z_index: u8) -> Self {
        Self { style, z_index }
    }
}
