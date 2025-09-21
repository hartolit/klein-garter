use std::collections::{HashMap, HashSet};

mod cell;
mod terrain;

use crate::core::global::{Id, Position, SlotMap};
use crate::prelude::{Glyph, Object, Occupant, TCell};
pub use cell::{Cell, CellRef};
pub use terrain::Terrain;

#[derive(Debug)]
pub struct SpatialGrid {
    cells: Vec<Cell>,
    empty_cells: SlotMap<usize>,
    pub width: u16,
    pub height: u16,
    pub border_style: Option<Glyph>,
    origin: Position, // The top left corner of the grid in world coordinates
    //is_bounded: bool, // TODO - Make bounding toggle
}

impl SpatialGrid {
    pub fn new<F>(width: u16, height: u16, border_style: Option<Glyph>, origin: Position, mut terrain_generator: F) -> Self
    where
        F: FnMut(Position) -> Terrain,
    {
        let size = height * width;
        let mut cells = Vec::with_capacity(size as usize);
        let mut empty_cells = SlotMap::new();

        for y in 0..height {
            for x in 0..width {
                let terrain = terrain_generator(Position::new(x, y));
                cells.push(Cell::new(terrain));
                let index = (y * width + x) as usize;
                empty_cells.insert(index);
            }
        }

        SpatialGrid {
            cells,
            empty_cells,
            width,
            height,
            border_style,
            origin,
            //is_bounded: false, // TODO - Make bounding toggle
        }
    }

    // pub fn toggle_bounds(&mut self) {
    //     self.is_bounded ^= true;
    // }

    pub fn pos_to_grid(&self, world_pos: Position) -> Option<Position> {
        let grid_x = world_pos.x.checked_sub(self.origin.x)?;
        let grid_y = world_pos.y.checked_sub(self.origin.y)?;

        if grid_x < self.width && grid_y < self.height {
            Some(Position { x: grid_x, y: grid_y })
        } else {
            None
        }
    }

    pub fn pos_to_world(&self, grid_pos: Position) -> Position {
        Position {
            x: self.origin.x + grid_pos.x,
            y: self.origin.y + grid_pos.y,
        }
    }

    pub fn get_border(&self) -> Vec<(Position, Glyph)> {
        let Some(glyph) = self.border_style else {
            return Vec::new();
        };

        let mut border_elements = Vec::new();
        let top_y = self.origin.y.saturating_sub(1);
        let bottom_y = self.origin.y + self.height;
        let left_x = self.origin.x.saturating_sub(1);
        let right_x = self.origin.x + self.width;

        // Top & bottom borders
        for x in left_x..=right_x {
            border_elements.push((Position::new(x, top_y), glyph));
            border_elements.push((Position::new(x, bottom_y), glyph));
        }

        // Left & right borders
        for y in (top_y + 1)..bottom_y {
            border_elements.push((Position::new(left_x, y), glyph));
            border_elements.push((Position::new(right_x, y), glyph));
        }

        border_elements
    }

    pub fn get_index(&self, grid_pos: &Position) -> Option<usize> {
        if grid_pos.x < self.width && grid_pos.y < self.height {
            Some((grid_pos.y * self.width + grid_pos.x) as usize)
        } else {
            None
        }
    }

    pub fn get_pos_from_index(&self, index: usize) -> Option<Position> {
        if index < self.cells.len() {
            let x = (index % self.width as usize) as u16;
            let y = (index / self.width as usize) as u16;
            Some(Position::new(x, y))
        } else {
            None
        }
    }

    pub fn get_cell(&self, world_pos: &Position) -> Option<&Cell> {
        self.pos_to_grid(*world_pos)
            .and_then(|grid_pos| self.get_index(&grid_pos))
            .map(|index| &self.cells[index])
    }

    pub fn get_cell_mut(&mut self, world_pos: &Position) -> Option<&mut Cell> {
        if let Some(grid_pos) = self.pos_to_grid(*world_pos) {
            if let Some(index) = self.get_index(&grid_pos) {
                return Some(&mut self.cells[index]);
            }
        }
        None
    }
    
    pub fn check_bounds(&self, object: &Box<dyn Object>) -> bool {
        for t_cell in object.t_cells() {
            if self.pos_to_grid(t_cell.pos).is_none() {
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
            .filter_map(|(id, world_pos)| {
                self.get_cell(&world_pos)
                    .map(|cell| (id, CellRef::new(world_pos, cell)))
            })
            .fold(HashMap::new(), |mut map, (id, cell_ref)| {
                map.entry(id).or_default().push(cell_ref);
                map
            })
    }
    
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

    pub fn remove_cell_occ(&mut self, occ: Occupant, world_pos: Position) -> bool {
        if let Some(grid_pos) = self.pos_to_grid(world_pos) {
            if let Some(index) = self.get_index(&grid_pos) {
                if let Some(t_cell) = self.cells[index].occ_by {
                    if occ == t_cell.occ {
                        self.cells[index].occ_by = None;
                        self.empty_cells.insert(index);
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn add_cell_occ(&mut self, t_cell: &TCell) -> bool {
        if let Some(grid_pos) = self.pos_to_grid(t_cell.pos) {
            if let Some(index) = self.get_index(&grid_pos) {
                 let (_, curr_z_index) = self.cells[index].top_glyph_and_z();
                if t_cell.z_index >= curr_z_index {
                    self.empty_cells.remove(&index);
                    self.cells[index].occ_by = Some(*t_cell);
                    return true;
                }
            }
        }
        false
    }

    pub fn random_empty_pos(&self) -> Option<Position> {
        self.empty_cells.get_random()
            .and_then(|index| self.get_pos_from_index(index))
            .map(|grid_pos| self.pos_to_world(grid_pos))
    }
}
