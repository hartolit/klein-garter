use crossterm::style::Color;
use rand::Rng;

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
    pub occ_by: Option<ObjectRef>,
    pub kind: CellKind,
}

impl GridCell {
    pub fn new(kind: CellKind) -> Self {
        GridCell { occ_by: None, kind }
    }
}

pub struct SpatialGrid {
    cells: Vec<GridCell>,
    pub full_width: u16,
    pub full_height: u16,
    pub game_width: u16,
    pub game_height: u16,
    pub border: u16,
}

impl SpatialGrid {
    pub fn new(game_width: u16, game_height: u16, mut border: u16, kind: CellKind) -> Self {
        if border < 1 {
            border = 1
        }

        let full_width = game_width + border * 2;
        let full_height = game_height + border * 2;
        let full_size = full_height * full_width;

        let mut cells = vec![GridCell::new(kind); full_size as usize];

        for (index, cell) in cells.iter_mut().enumerate() {
            let x = index % full_width as usize;
            let y = index / full_height as usize;

            if x < (border as usize)
                || x >= (game_width + border) as usize
                || y < (border as usize)
                || y >= (game_height + border) as usize
            {
                cell.kind = CellKind::Border;
            }
        }

        SpatialGrid {
            cells: cells,
            full_width,
            full_height,
            game_width,
            game_height,
            border,
        }
    }

    pub fn get_index(&self, pos: Position) -> Option<usize> {
        if pos.x < self.full_width && pos.y < self.full_height {
            Some((pos.y * self.full_width + pos.x) as usize)
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
                // TODO - handle collisions?
                if cell.occ_by.is_none() {
                    cell.occ_by = Some(obj_ref);
                }
            }
        }
    }

    pub fn remove_object(&mut self, positions: &[Position]) {
        for &pos in positions {
            if let Some(cell) = self.get_cell_mut(pos) {
                cell.occ_by = None;
            }
        }
    }

    // TODO - make better
    pub fn rng_empty_pos(&self, offset: Option<u16>) -> Position {
        let off = offset.unwrap_or(0);
        let mut pos = Position::new(0, 0);

        // !DEAD LOOP WHEN NO EMPTY SPOTS ARE FOUND
        loop {
            pos.x = rand::rng().random_range(self.border + off..self.game_width - off);
            pos.y = rand::rng().random_range(self.border + off..self.game_height - off);

            if let Some(cell) = self.get_cell(pos) {
                if cell.occ_by.is_none() {
                    break;
                }
            };
        }

        pos
    }

    pub fn move_object(
        &mut self,
        obj_ref: ObjectRef,
        old_positions: &[Position],
        new_positions: &[Position],
    ) {
        self.remove_object( old_positions);
        self.add_object(obj_ref, new_positions);
    }
}
