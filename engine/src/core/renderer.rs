use std::io::{self, Stdout};
use crossterm::{self, style::{SetBackgroundColor, SetForegroundColor}, QueueableCommand};

use crate::core::object::{state::{StateChange, StateManager}, element::{Glyph}};
use crate::core::global::Position;
use super::grid::SpatialGrid;

pub struct Renderer {
    stdout: Stdout,
}

impl Renderer {
    pub fn draw(&mut self, spatial_grid: &SpatialGrid, global_state: &mut StateManager) {
        for (_, state) in global_state.changes.drain() {
            match state {
                StateChange::Create { new_element, .. } => {
                    self.draw_glyph(new_element.style, new_element.pos).unwrap();
                },
                StateChange::Delete { init_pos, .. } => {
                    let cell = spatial_grid.get_cell(init_pos);
                    if let Some(cell) = cell {
                        let cell_glyph = cell.kind.appearance();
                        self.draw_glyph(cell_glyph, init_pos).unwrap();
                    }
                },
                StateChange::Update { element, init_pos, ..} => {
                    let cell = spatial_grid.get_cell(init_pos);
                    if let Some(cell) = cell {
                        let cell_glyph = cell.kind.appearance();
                        self.draw_glyph(cell_glyph, init_pos).unwrap();
                        self.draw_glyph(element.style, element.pos).unwrap();
                    }
                },
            }
        }
    }

    pub fn draw_glyph(&mut self, glyph: Glyph, pos: Position) -> io::Result<()> {
        if let Some(fg_color) = glyph.fg_clr {
            self.stdout.queue(SetForegroundColor(fg_color))?;
        }

        if let Some(bg_color) = glyph.bg_clr {
            self.stdout.queue(SetBackgroundColor(bg_color))?;
        }

        self.stdout
            .queue(crossterm::cursor::MoveTo(pos.x, pos.y))?
            .queue(crossterm::style::Print(glyph.symbol))?;

        Ok(())
    }
}