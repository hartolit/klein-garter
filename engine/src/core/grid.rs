use rand::Rng;

pub mod cell;
use super::global::Position;
use super::object::{Object, state::Occupant};
use cell::{Cell, Kind};

pub struct SpatialGrid {
    cells: Vec<Cell>,
    pub full_width: u16,
    pub full_height: u16,
    pub game_width: u16,
    pub game_height: u16,
    pub border: u16,
}

impl SpatialGrid {
    pub fn new(game_width: u16, game_height: u16, mut border: u16, kind: Kind) -> Self {
        if border < 1 {
            border = 1
        }

        let full_width = game_width + border * 2;
        let full_height = game_height + border * 2;
        let full_size = full_height * full_width;

        let mut cells = vec![Cell::new(kind); full_size as usize];

        for (index, cell) in cells.iter_mut().enumerate() {
            let x = index % full_width as usize;
            let y = index / full_width as usize;

            if x < (border as usize)
                || x >= (game_width + border) as usize
                || y < (border as usize)
                || y >= (game_height + border) as usize
            {
                cell.kind = Kind::Border;
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

    pub fn get_pos_from_index(&self, index: usize) -> Option<Position> {
        if index < self.cells.len() {
            let x = (index % self.full_width as usize) as u16;
            let y = (index / self.full_width as usize) as u16;
            Some(Position::new(x, y))
        } else {
            None
        }
    }

    pub fn get_cell(&self, pos: Position) -> Option<&Cell> {
        self.get_index(pos).map(|index| &self.cells[index])
    }

    pub fn get_cell_mut(&mut self, pos: Position) -> Option<&mut Cell> {
        self.get_index(pos).map(move |index| &mut self.cells[index])
    }

    pub fn add_object<T: Object>(&mut self, object: &T) {
        for element in object.elements() {
            let cell = match self.get_cell_mut(element.pos) {
                Some(cell) => cell,
                None => continue,
            };
            // ! Overwrites current occupant
            cell.occ_by = Some(Occupant::new(object.id(), element.id));
        }
    }

    pub fn remove_object<T: Object>(&mut self, object: &T) {
        for element in object.elements() {
            let cell = match self.get_cell_mut(element.pos) {
                Some(cell) => cell,
                None => continue,
            };

            if let Some(occ) = cell.occ_by {
                if occ.obj_id == object.id() {
                    cell.occ_by = None
                }
            }
        }
    }

    pub fn remove_cell_occ(&mut self, occ: Occupant, pos: Position) {
        let cell = match self.get_cell_mut(pos) {
            Some(cell) => cell,
            None => return,
        };

        if let Some(cell_occ) = cell.occ_by {
            if occ == cell_occ {
                cell.occ_by = None;
            }
        }
    }

    pub fn add_cell_occ(&mut self, occ: Occupant, pos: Position) {
        let cell = match self.get_cell_mut(pos) {
            Some(cell) => cell,
            None => return,
        };
        // ! Overwrites current occupant
        cell.occ_by = Some(occ);
    }

    pub fn rng_empty_pos(&self) -> Option<Position> {
        let empty_pos: Vec<usize> = self
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.occ_by.is_none())
            .map(|(index, _)| index)
            .collect();

        if empty_pos.is_empty() {
            return None;
        }

        let rnd_pos = rand::rng().random_range(0..empty_pos.len());

        let pos = match empty_pos.get(rnd_pos) {
            Some(pos) => pos,
            None => return None,
        };

        self.get_pos_from_index(*pos)
    }
}
