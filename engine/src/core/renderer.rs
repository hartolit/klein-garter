use crossterm::{
    self, QueueableCommand, cursor, execute,
    style::{self, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::collections::HashMap;
use std::io::{Stdout, Write, stdout};

use super::grid::SpatialGrid;
use super::scene::global_state::CategorizedStates;
use crate::core::global::{Id, Position};
use crate::core::object::{Object, state::StateChange, t_cell::Glyph};

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

    pub fn full_render(
        &mut self,
        grid: &Option<SpatialGrid>,
        objects: &HashMap<Id, Box<dyn Object>>,
    ) {
        let mut glyph_map: HashMap<Position, &Glyph> = HashMap::new();
        for object in objects.values() {
            for t_cell in object.t_cells() {
                glyph_map.insert(t_cell.pos, &t_cell.style);
            }
        }

        

        for y in 0..grid.full_height {
            for x in 0..grid.full_width {
                let pos = Position::new(x, y);

                let glyph = if let Some(glyph) = glyph_map.get(&pos) {
                    *glyph
                } else {
                    let index = (y * grid.full_width + x) as usize;
                    &grid.cells[index].kind.appearance()
                };

                self.draw_glyph(glyph, &pos);
            }
        }

        self.stdout.flush().unwrap();
    }

    pub fn partial_render(
        &mut self,
        grid: &Option<SpatialGrid>,
        filtered_states: &CategorizedStates,
    ) {
        for state in filtered_states.deletes.iter() {
            if let StateChange::Delete { init_pos, .. } = state {
                if let Some(cell) = grid.get_cell(init_pos) {
                    let cell_glyph = cell.kind.appearance();
                    self.draw_glyph(&cell_glyph, init_pos);
                } else {
                    self.clear_glyph(init_pos);
                }
            }
        }

        for state in filtered_states.updates.iter() {
            if let StateChange::Update {
                t_cell, init_pos, ..
            } = state
            {
                if &t_cell.pos != init_pos {
                    if let Some(cell) = grid.get_cell(init_pos) {
                        let cell_glyph = cell.kind.appearance();
                        self.draw_glyph(&cell_glyph, init_pos);
                    } else {
                        self.clear_glyph(init_pos);
                    }
                }
                self.draw_glyph(&t_cell.style, &t_cell.pos);
            }
        }

        for state in filtered_states.creates.iter() {
            if let StateChange::Create { new_t_cell, .. } = state {
                self.draw_glyph(&new_t_cell.style, &new_t_cell.pos);
            }
        }

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
        self.stdout
            .queue(cursor::MoveTo(pos.x, pos.y))
            .unwrap()
            .queue(ResetColor)
            .unwrap()
            .queue(style::Print(' '))
            .unwrap();
    }
}
