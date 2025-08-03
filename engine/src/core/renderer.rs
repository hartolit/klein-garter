use crossterm::{
    self, QueueableCommand, cursor, execute,
    style::{SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::io::{Stdout, stdout};

use super::grid::SpatialGrid;
use crate::core::global::Position;
use crate::core::object::{
    element::Glyph,
    state::{StateChange, StateManager},
};

pub struct Renderer {
    stdout: Stdout,
}

impl Renderer {
    pub fn new() -> Self {
        Self { stdout: stdout() }
    }

    pub fn init(&mut self) {
        terminal::enable_raw_mode().unwrap();
        execute!(self.stdout, cursor::Hide).unwrap();
    }

    pub fn kill(&mut self) {
        terminal::disable_raw_mode().unwrap();
        execute!(self.stdout, cursor::Show).unwrap();
    }

    pub fn draw(&mut self, spatial_grid: &SpatialGrid, global_state: &mut StateManager) {
        let max_len = global_state.changes.len();
        let mut updates: Vec<StateChange> = Vec::with_capacity(max_len);
        let mut creates: Vec<StateChange> = Vec::with_capacity(max_len);

        for (_, state) in global_state.changes.drain() {
            match state {
                StateChange::Delete { init_pos, .. } => {
                    if let Some(cell) = spatial_grid.get_cell(init_pos) {
                        let cell_glyph = cell.kind.appearance();
                        self.draw_glyph(cell_glyph, init_pos);
                    }
                }
                StateChange::Update { .. } => updates.push(state),
                StateChange::Create { .. } => creates.push(state),
            }
        }

        for state in updates {
            if let StateChange::Update {
                element, init_pos, ..
            } = state
            {
                if element.pos != init_pos {
                    if let Some(cell) = spatial_grid.get_cell(init_pos) {
                        let cell_glyph = cell.kind.appearance();
                        self.draw_glyph(cell_glyph, init_pos);
                    }
                }
                self.draw_glyph(element.style, element.pos);
            }
        }

        for state in creates {
            if let StateChange::Create { new_element, .. } = state {
                self.draw_glyph(new_element.style, new_element.pos);
            }
        }
    }

    pub fn draw_glyph(&mut self, glyph: Glyph, pos: Position) {
        if let Some(fg_color) = glyph.fg_clr {
            self.stdout.queue(SetForegroundColor(fg_color)).unwrap();
        }

        if let Some(bg_color) = glyph.bg_clr {
            self.stdout.queue(SetBackgroundColor(bg_color)).unwrap();
        }

        self.stdout
            .queue(crossterm::cursor::MoveTo(pos.x, pos.y))
            .unwrap()
            .queue(crossterm::style::Print(glyph.symbol))
            .unwrap();
    }
}
