use std::collections::{HashMap, HashSet};

pub mod cell;
pub mod terrain;

use crate::prelude::TCell;

use super::global::{Id, Position, SlotMap};
use super::object::{Object, Occupant};
use cell::{Cell, CellRef};
use terrain::Terrain;

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
    pub fn new<F>(
        game_width: u16,
        game_height: u16,
        mut border: u16,
        mut terrain_generator: F,
    ) -> Self
    where
        F: FnMut(Position, bool) -> Terrain,
    {
        if border < 1 {
            border = 1
        }

        let full_width = game_width + border * 2;
        let full_height = game_height + border * 2;
        let full_size = full_height * full_width;

        let mut cells = Vec::with_capacity(full_size as usize);
        let mut empty_cells = SlotMap::new();

        for y in 0..full_height {
            for x in 0..full_width {
                let pos = Position::new(x, y);
                let is_border = x < border
                    || x >= game_width + border
                    || y < border
                    || y >= game_height + border;

                let terrain = terrain_generator(pos, is_border);
                cells.push(Cell::new(terrain));

                if !is_border {
                    let index = (y * full_width + x) as usize;
                    empty_cells.insert(index);
                }
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

    /// Checks an objects bounds within the game area
    pub fn check_bounds(&self, object: &Box<dyn Object>) -> bool {
        for t_cell in object.t_cells() {
            if !self.is_within_game_area(&t_cell.pos) {
                return false;
            }
        }
        true
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

    /// Probes an object and gets a vec of collided object Ids
    pub fn probe_object(&self, object: &Box<dyn Object>) -> HashSet<Id> {
        let mut collision_ids: HashSet<Id> = HashSet::new();
        for t_cell in object.t_cells() {
            if let Some(cell) = self.get_cell(&t_cell.pos) {
                if let Some(occupant_t_cell) = &cell.occ_by {
                    if occupant_t_cell.occ.obj_id != object.id() {
                        collision_ids.insert(occupant_t_cell.occ.obj_id);
                    }
                }
            }
        }

        collision_ids
    }

    pub fn add_object(&mut self, object: &Box<dyn Object>) {
        for t_cell in object.t_cells() {
            self.add_cell_occ(t_cell);
        }
    }

    pub fn remove_object(&mut self, object: &Box<dyn Object>) {
        for t_cell in object.t_cells() {
            self.remove_cell_occ(t_cell.occ, t_cell.pos);
        }
    }

    pub fn remove_cell_occ(&mut self, occ: Occupant, pos: Position) -> bool {
        if !self.is_within_game_area(&pos) {
            return false;
        }

        if let Some(global_index) = self.get_index(&pos) {
            if let Some(t_cell) = self.cells[global_index].occ_by {
                if occ == t_cell.occ {
                    self.cells[global_index].occ_by = None;
                    self.empty_cells.insert(global_index);
                    return true;
                }
            }
        }

        return false;
    }

    pub fn add_cell_occ(&mut self, t_cell: &TCell) -> bool {
        if !self.is_within_game_area(&t_cell.pos) {
            return false;
        }

        if let Some(global_index) = self.get_index(&t_cell.pos) {
            let (_, curr_z_index) = self.cells[global_index].top_glyph_and_z();
            if t_cell.z_index >= curr_z_index {
                self.empty_cells.remove(&global_index);
                self.cells[global_index].occ_by = Some(*t_cell);
                return true;
            }
        }

        return false;
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
