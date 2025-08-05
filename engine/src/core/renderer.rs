use crossterm::{
    self, QueueableCommand, cursor, execute,
    style::{SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::io::{Stdout, Write, stdout};

use super::grid::SpatialGrid;
use super::world::global_state::CategorizedStates;
use crate::core::global::Position;
use crate::core::object::{element::Glyph, state::StateChange};

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

    // TODO - Make SpatialGrid + Objects an iterator
    // TODO - Make SpatialGrid an iterator
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

        self.stdout.flush().unwrap();
    }

    pub fn draw_glyph(&mut self, glyph: &Glyph, pos: &Position) {
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
