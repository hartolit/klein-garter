use crossterm::style::Color;

use crate::game::food::{ self};

use super::object::{ObjectId, Glyph, Position};

#[derive(Debug, Clone, Copy)]
pub enum ObjectRef {
    Player(ObjectId),
    Food(ObjectId, food::Kind, i16), // Last type is amount of meals
}

// PartialEq for ObjectId only
impl PartialEq for ObjectRef {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjectRef::Player(id1), ObjectRef::Player(id2)) => id1 == id2,
            (ObjectRef::Food(id1, _, _), ObjectRef::Food(id2, _, _)) => id1 == id2,
            _ => false,
        }
    }
}

impl Eq for ObjectRef {}

#[derive(Debug, Clone, Copy)]
pub enum CellKind {
    Ground,
    Water,
    Lava,
    Border,
}

impl CellKind {
    pub fn appearance(&self) -> Glyph {
        match self {
            CellKind::Ground => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::Black),
                symbol: ' '
            },
            CellKind::Water => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkBlue),
                symbol: '≈'
            },
            CellKind::Lava => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkRed),
                symbol: '#'
            },
            CellKind::Border => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkGrey),
                symbol: '█'
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridCell {
    pub occ_by: Vec<ObjectRef>,
    pub kind: CellKind
}

impl GridCell {
    pub fn new(kind: CellKind) -> Self {
        GridCell {
            occ_by: Vec::new(),
            kind
        }
    }
}

pub struct SpatialGrid {
    cells: Vec<GridCell>,
    pub width: u16,
    pub height: u16,
}

impl SpatialGrid {
    pub fn new(width: u16, height: u16, kind: CellKind) -> Self {
        let size = (width * height) as usize;
        let cells = vec![GridCell::new(kind); size];
        SpatialGrid { cells: cells, width, height }
    }

    pub fn get_index(&self, pos: Position) -> Option<usize> {
        if pos.x < self.width && pos.y < self.height {
            Some((pos.y * self.width + pos.x) as usize)
        } else {
            None
        }
    }

    pub fn get_cell(&self, pos: Position) -> Option<&GridCell> {
        self.get_index(pos).map(|index| &self.cells[index])
    }

    pub fn get_cell_mut(&mut self, pos: Position) -> Option<&mut GridCell> {
        self.get_index(pos).map(move |index| &mut self.cells[index])
    }

    pub fn add_object(&mut self, obj_ref: ObjectRef, positions: &[Position]) {
        for &pos in positions {
            if let Some(cell) = self.get_cell_mut(pos) {
                cell.occ_by.push(obj_ref);
            }
        }
    }

    pub fn remove_object(&mut self, obj_ref: &ObjectRef, positions: &[Position]) {
        for &pos in positions {
            if let Some(cell) = self.get_cell_mut(pos) {
                cell.occ_by.retain(|r| r != obj_ref);
            }
        }
    }

    pub fn move_object(&mut self, obj_ref: ObjectRef, old_positions: &[Position], new_positions: &[Position]) {
        self.remove_object(&obj_ref, old_positions);
        self.add_object(obj_ref, new_positions);
    }
}