use crossterm::style::Color;

use engine::prelude::*;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Logger {
    id: Id,
    id_counter: IdCounter,
    state: State,
    t_cells: Vec<TCell>,
    pub pos: Position,
    log_messages: VecDeque<String>,
    max_lines: usize,
}

impl Logger {
    pub fn new(id: Id, pos: Position, max_lines: usize) -> Self {
        Self {
            id,
            id_counter: IdCounter::new(),
            state: State::new(),
            t_cells: Vec::new(),
            pos,
            log_messages: VecDeque::with_capacity(max_lines),
            max_lines,
        }
    }

    pub fn clear(&mut self) {
        self.log_messages.clear();
        self.t_cells.clear();
    }

    pub fn add_log(&mut self, message: String, fg_clr: Option<Color>) {
        if self.log_messages.len() == self.max_lines {
            self.log_messages.pop_front();
        }
        self.log_messages.push_back(message);
        self.update_display(fg_clr);
    }

    fn update_display(&mut self, fg_clr: Option<Color>) {
        for t_cell in self.t_cells.drain(..) {
            self.state.upsert_change(StateChange::Delete {
                occupant: t_cell.occ,
                init_pos: t_cell.pos,
            });
        }

        for (y, line) in self.log_messages.iter().enumerate() {
            for (x, character) in line.chars().enumerate() {
                let t_cell = TCell::new(
                    Occupant::new(self.id, self.id_counter.next()),
                    Glyph::new(fg_clr, None, character),
                    Some(Position::new(self.pos.x + x as u16, self.pos.y + y as u16)),
                    255,
                );
                self.t_cells.push(t_cell);
                self.state
                    .upsert_change(StateChange::Create { new_t_cell: t_cell });
            }
        }
    }

    fn get_t_cells(&self) -> Box<dyn Iterator<Item = &TCell> + '_> {
        Box::new(self.t_cells.iter())
    }
}

define_object! {
    struct Logger,
    id_field: id,
    t_cells: custom(get_t_cells),
    capabilities: {
        Stateful { state_field: state }
        Destructible {}
    }
}
