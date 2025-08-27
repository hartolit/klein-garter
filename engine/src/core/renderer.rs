use crossterm::{
    self, QueueableCommand, cursor, execute, style,
    style::{SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::collections::HashMap;
use std::io::{Stdout, Write, stdout};

use super::grid::SpatialGrid;
use super::scene::global_state::CategorizedStates;
use crate::core::global::{Id, Position};
use crate::core::object::{Object, t_cell::Glyph, state::StateChange};

pub struct Renderer {
    stdout: Stdout,
}

impl Renderer {
    pub fn new() -> Self {
        let mut stdout = stdout();

        terminal::enable_raw_mode().unwrap();
        execute!(stdout, cursor::Hide).unwrap();

        Self { stdout }
    }

    pub fn kill(&mut self) {
        terminal::disable_raw_mode().unwrap();
        execute!(self.stdout, cursor::Show).unwrap();
    }

    // TODO - Add indexing to handle overlapping elements
    pub fn full_render(
        &mut self,
        spatial_grid: &mut SpatialGrid,
        objects: &HashMap<Id, Box<dyn Object>>,
    ) {
        let mut glyph_map: HashMap<Position, &Glyph> = HashMap::new();
        for object in objects.values() {
            for element in object.elements() {
                glyph_map.insert(element.pos, &element.style);
            }
        }

        for y in 0..spatial_grid.full_height {
            for x in 0..spatial_grid.full_width {
                let pos = Position::new(x, y);

                let glyph = if let Some(glyph) = glyph_map.get(&pos) {
                    *glyph
                } else {
                    let index = (y * spatial_grid.full_width + x) as usize;
                    &spatial_grid.cells[index].kind.appearance()
                };

                self.draw_glyph(glyph, &pos);
            }
        }

        // TODO - Add rendering outside spatialgrid
        self.stdout.flush().unwrap();
    }

    pub fn partial_render(
        &mut self,
        spatial_grid: &SpatialGrid,
        finalized_state: &CategorizedStates,
    ) {
        for state in finalized_state.deletes.iter() {
            if let StateChange::Delete { init_pos, .. } = state {
                if let Some(cell) = spatial_grid.get_cell(init_pos) {
                    let cell_glyph = cell.kind.appearance();
                    self.draw_glyph(&cell_glyph, init_pos);
                }
            }
        }

        for state in finalized_state.updates.iter() {
            if let StateChange::Update {
                element, init_pos, ..
            } = state
            {
                if &element.pos != init_pos {
                    if let Some(cell) = spatial_grid.get_cell(init_pos) {
                        let cell_glyph = cell.kind.appearance();
                        self.draw_glyph(&cell_glyph, init_pos);
                    }
                }
                self.draw_glyph(&element.style, &element.pos);
            }
        }

        for state in finalized_state.creates.iter() {
            if let StateChange::Create { new_element, .. } = state {
                self.draw_glyph(&new_element.style, &new_element.pos);
            }
        }

        // TODO - Add rendering outside spatialgrid

        self.stdout.flush().unwrap();
    }

    fn draw_glyph(&mut self, glyph: &Glyph, pos: &Position) {
        if let Some(fg_color) = glyph.fg_clr {
            self.stdout.queue(SetForegroundColor(fg_color)).unwrap();
        }

        if let Some(bg_color) = glyph.bg_clr {
            self.stdout.queue(SetBackgroundColor(bg_color)).unwrap();
        }

        self.stdout
            .queue(cursor::MoveTo(pos.x, pos.y))
            .unwrap()
            .queue(style::Print(glyph.symbol))
            .unwrap();
    }

    fn clear_glyph(&mut self, pos: &Position) {
        self.stdout.queue(cursor::MoveTo(pos.x, pos.y)).unwrap().queue(style::Print(' ')).unwrap();
    }
    
}
