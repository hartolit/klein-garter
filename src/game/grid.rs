use std::cell::Cell;

use crossterm::style::Color;

use crate::game::food::{self};

use super::object::{Glyph, Id, Position};

#[derive(Debug, Clone, Copy)]
pub enum ObjectRef {
    Player(Id),
    Food {
        obj_id: Id,
        elem_id: Id,
        kind: food::Kind,
        meals: i16,
    },
}

// PartialEq for ObjectId only
impl PartialEq for ObjectRef {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjectRef::Player(id1), ObjectRef::Player(id2)) => id1 == id2,
            (
                ObjectRef::Food {
                    obj_id: id1,
                    kind: _,
                    meals: _,
                    elem_id: _,
                },
                ObjectRef::Food {
                    obj_id: id2,
                    kind: _,
                    meals: _,
                    elem_id: _,
                },
            ) => id1 == id2,
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
                symbol: ' ',
            },
            CellKind::Water => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkBlue),
                symbol: '≈',
            },
            CellKind::Lava => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkRed),
                symbol: '#',
            },
            CellKind::Border => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkGrey),
                symbol: '█',
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridCell {
    pub occ_by: Vec<ObjectRef>,
    pub kind: CellKind,
}

impl GridCell {
    pub fn new(kind: CellKind) -> Self {
        GridCell {
            occ_by: Vec::new(),
            kind,
        }
    }
}

pub struct SpatialGrid {
    cells: Vec<GridCell>,
    pub width: u16,
    pub height: u16,
    pub border: u16,
}

impl SpatialGrid {
    pub fn new(inner_width: u16, inner_height: u16, mut border: u16, kind: CellKind) -> Self {
        if border < 1 {
            border = 1
        }

        let total_width = inner_width + border * 2;
        let total_height = inner_height + border * 2;
        let total_size = total_height * total_width;

        let mut cells = vec![GridCell::new(kind); total_size as usize];

        for (index, cell) in cells.iter_mut().enumerate() {
            let x = index % total_width as usize;
            let y = index / total_height as usize;

            if x < (border as usize)
                || x >= (inner_width + border) as usize
                || y < (border as usize)
                || y >= (inner_height + border) as usize
            {
                cell.kind = CellKind::Border;
            }
        }

        SpatialGrid {
            cells: cells,
            width: total_width,
            height: total_height,
            border,
        }
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

    // TODO - create safe random position
    // pub fn rng_pos(&self, offset: Option<u16>) -> Position {
    //     let off = offset.unwrap_or(0);
    //     let x =
    //         rand::rng().random_range(self.border_width + off..self.width + self.border_width - off);
    //     let y = rand::rng()
    //         .random_range(self.border_height + off..self.height + self.border_height - off);

    //     Position { x: x, y: y }
    // }

    pub fn move_object(
        &mut self,
        obj_ref: ObjectRef,
        old_positions: &[Position],
        new_positions: &[Position],
    ) {
        self.remove_object(&obj_ref, old_positions);
        self.add_object(obj_ref, new_positions);
    }
}
