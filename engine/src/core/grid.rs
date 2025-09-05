use std::collections::HashMap;

pub mod cell;

use super::global::{Id, Position, SlotMap};
use super::object::{Object, Occupant};
use cell::{Cell, CellRef, Kind};

#[derive(Debug)]
pub struct SpatialGrid {
    pub cells: Vec<Cell>,
    pub empty_cells: SlotMap<usize>,
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

        let mut empty_cells = SlotMap::new();
        for y in border..(game_height + border) {
            for x in border..(game_width + border) {
                let index = (y * full_width + x) as usize;
                empty_cells.insert(index);
            }
        }

        SpatialGrid {
            cells,
            empty_cells,
            full_width,
            full_height,
            game_width,
            game_height,
            border,
        }
    }

    pub fn get_index(&self, pos: &Position) -> Option<usize> {
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

    pub fn iter_cell(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }

    pub fn get_cell(&self, pos: &Position) -> Option<&Cell> {
        self.get_index(pos).map(|index| &self.cells[index])
    }

    pub fn get_cell_mut(&mut self, pos: &Position) -> Option<&mut Cell> {
        self.get_index(pos).map(move |index| &mut self.cells[index])
    }

    pub fn probe_moves<'a>(
        &'a self,
        moves: impl Iterator<Item = (Id, Position)>,
    ) -> HashMap<Id, Vec<CellRef<'a>>> {
        moves
            .filter_map(|(id, pos)| {
                self.get_cell(&pos)
                    .map(|cell| (id, CellRef::new(pos, cell)))
            })
            .fold(HashMap::new(), |mut map, (id, cell_ref)| {
                map.entry(id).or_default().push(cell_ref);
                map
            })
    }

    // pub fn check_bounds(&self, object: &Box<dyn Object>) -> bool {
    //     for t_cell in object.t_cells() {
    //         if let None = self.get_cell(&t_cell.pos) {
    //             return false
    //         }
    //     }
    //     true
    // }

    pub fn check_object_placement(&mut self, object: &Box<dyn Object>, objects: &HashMap<Id, Box<dyn Object>>) -> bool {
        let new_z = object.z_index();

        for t_cell in object.t_cells() {
            if let Some(cell) = self.get_cell(&t_cell.pos) {
                if let Some(existing_occupant) = cell.occ_by {
                    if let Some(existing_object) = objects.get(&existing_occupant.obj_id) {
                        if new_z < existing_object.z_index() {
                            return false;
                        }
                    }
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn add_object(&mut self, object: &Box<dyn Object>) {
        for t_cell in object.t_cells() {
            self.add_cell_occ(t_cell.occ, t_cell.pos);
        }
    }

    pub fn remove_object(&mut self, object: &Box<dyn Object>) {
        for t_cell in object.t_cells() {
            self.remove_cell_occ(t_cell.occ, t_cell.pos);
        }
    }

    pub fn remove_cell_occ(&mut self, occ: Occupant, pos: Position) {
        if !self.is_within_game_area(&pos) {
            return;
        }
        if let Some(global_index) = self.get_index(&pos) {
            if let Some(cell_occ) = self.cells[global_index].occ_by {
                if occ == cell_occ {
                    self.cells[global_index].occ_by = None;
                    self.empty_cells.insert(global_index);
                }
            }
        }
    }

    pub fn add_cell_occ(&mut self, occ: Occupant, pos: Position) {
        if !self.is_within_game_area(&pos) {
            return;
        }
        if let Some(global_index) = self.get_index(&pos) {
            if self.cells[global_index].occ_by.is_none() {
                self.empty_cells.remove(&global_index);
                self.cells[global_index].occ_by = Some(occ);
            }
        }
    }

    pub fn random_empty_pos(&self) -> Option<Position> {
        let random_pos = match self.empty_cells.get_random() {
            Some(index) => self.get_pos_from_index(index),
            None => None,
        };

        random_pos
    }

    pub fn is_within_game_area(&self, pos: &Position) -> bool {
        pos.x >= self.border
            && pos.x < self.game_width + self.border
            && pos.y >= self.border
            && pos.y < self.game_height + self.border
    }
}
