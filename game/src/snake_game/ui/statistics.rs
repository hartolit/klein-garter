use crossterm::style::Color;
use engine::prelude::*;

#[derive(Debug)]
pub struct Statistics {
    id: Id,
    id_counter: IdCounter,
    state: State,
    t_cells: Vec<TCell>,
    pos: Position,
}

impl Statistics {
    pub fn new(id: Id, pos: Position) -> Self {
        Self {
            id,
            id_counter: IdCounter::new(),
            state: State::new(),
            t_cells: Vec::new(),
            pos,
        }
    }

    pub fn set_text(&mut self, lines: Vec<String>) {
        for t_cell in self.t_cells.drain(..) {
            self.state.upsert_change(StateChange::Delete {
                occupant: t_cell.occ,
                init_pos: t_cell.pos,
            });
        }

        for (y, line) in lines.iter().enumerate() {
            for (x, character) in line.chars().enumerate() {
                let t_cell = TCell::new(
                    Occupant::new(self.id, self.id_counter.next()),
                    Glyph::new(Some(Color::White), Some(Color::Black), character),
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
    struct Statistics,
    id_field: id,
    t_cells: custom(get_t_cells),
    capabilities: {
        Stateful { state_field: state }
        Destructible {}
    }
}
