use crate::prelude::{Glyph, Position, TCell};
use super::terrain::Terrain;

#[derive(Debug, Clone)]
pub struct Cell {
    pub occ_by: Option<TCell>,
    pub terrain: Terrain,
}

impl Cell {
    pub fn new(terrain: Terrain) -> Self {
        Cell {
            occ_by: None,
            terrain,
        }
    }

    pub fn top_glyph_and_z(&self) -> (&Glyph, u8) {
        if let Some(occ) = &self.occ_by {
            if occ.z_index >= self.terrain.z_index {
                return (&occ.style, occ.z_index);
            }
        }
        (&self.terrain.style, self.terrain.z_index)
    }
}

pub struct CellRef<'a> {
    pub pos: Position,
    pub cell: &'a Cell,
}

impl<'a> CellRef<'a> {
    pub fn new(pos: Position, cell: &'a Cell) -> Self {
        CellRef { pos, cell }
    }
}
