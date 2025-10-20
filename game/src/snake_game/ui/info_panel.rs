use crossterm::style::Color;
use engine::prelude::*;

#[derive(Debug)]
pub struct InfoPanel {
    id: Id,
    id_counter: IdCounter,
    state: State,
    t_cells: Vec<TCell>,
    start_pos: Position,
    line_offset: u16,
}

impl InfoPanel {
    pub fn new(id: Id, pos: Position) -> Self {
        Self {
            id,
            id_counter: IdCounter::new(),
            state: State::new(),
            t_cells: Vec::new(),
            start_pos: pos,
            line_offset: 0,
        }
    }

    pub fn clear(&mut self) {
        for t_cell in self.t_cells.drain(..) {
            self.state.upsert_change(StateChange::Delete {
                occupant: t_cell.occ,
                init_pos: t_cell.pos,
            });
        }
        self.line_offset = 0;
    }

    pub fn add_line(&mut self, text: String, fg_clr: Option<Color>, bg_color: Option<Color>) {
        let line_y = self.start_pos.y + self.line_offset;

        for (i, ch) in text.chars().enumerate() {
            let t_cell = TCell::new(
                Occupant::new(self.id, self.id_counter.next()),
                Glyph::new(fg_clr, bg_color, ch),
                Some(Position::new(self.start_pos.x + i as u16, line_y)),
                255,
            );
            self.t_cells.push(t_cell);
            self.state
                .upsert_change(StateChange::Create { new_t_cell: t_cell });
        }
        self.line_offset += 1;
    }

    fn get_t_cells(&self) -> Box<dyn Iterator<Item = &TCell> + '_> {
        Box::new(self.t_cells.iter())
    }
}

define_object! {
    struct InfoPanel,
    id_field: id,
    t_cells: custom(get_t_cells),
    capabilities: {
        Stateful { state_field: state }
        Destructible {}
    }
}
