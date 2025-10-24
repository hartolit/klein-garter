use crossterm::style::Color;
use engine::prelude::*;
use std::mem;

#[derive(Debug)]
pub struct Statistics {
    id: Id,
    id_counter: IdCounter,
    state: State,
    t_cells_per_line: Vec<Vec<TCell>>,
    pub pos: Position,
    lines: Vec<String>,
}

impl Statistics {
    pub fn new(id: Id, pos: Position) -> Self {
        Self {
            id,
            id_counter: IdCounter::new(),
            state: State::new(),
            t_cells_per_line: Vec::new(),
            pos,
            lines: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.t_cells_per_line.clear();
        self.lines.clear();
    }

    pub fn set_text(&mut self, new_lines: Vec<String>, fg_clr: Option<Color>) {
        if self.lines == new_lines {
            return;
        }

        let old_line_count = self.lines.len();
        let new_line_count = new_lines.len();
        let max_lines = old_line_count.max(new_line_count);

        let mut new_t_cells_per_line = Vec::with_capacity(new_line_count);

        for y in 0..max_lines {
            let old_line = self.lines.get(y);
            let new_line = new_lines.get(y);

            match (old_line, new_line) {
                (Some(old), Some(new)) if old != new => {
                    let old_t_cells = mem::take(&mut self.t_cells_per_line[y]);
                    let updated_t_cells = self.update_line_by_char(y, old_t_cells, new, fg_clr);
                    new_t_cells_per_line.push(updated_t_cells);
                }
                (None, Some(new)) => {
                    let new_t_cells = self.create_line_by_char(y, new, fg_clr);
                    new_t_cells_per_line.push(new_t_cells);
                }
                (Some(_), None) => {
                    for t_cell in &self.t_cells_per_line[y] {
                        self.state.upsert_change(StateChange::Delete {
                            occupant: t_cell.occ,
                            init_pos: t_cell.pos,
                        });
                    }
                }
                (Some(_), Some(_)) => {
                    new_t_cells_per_line.push(mem::take(&mut self.t_cells_per_line[y]));
                }
                (None, None) => {}
            }
        }

        self.t_cells_per_line = new_t_cells_per_line;
        self.lines = new_lines;
    }

    fn create_line_by_char(
        &mut self,
        y: usize,
        new_line: &str,
        fg_clr: Option<Color>,
    ) -> Vec<TCell> {
        let mut t_cells = Vec::with_capacity(new_line.len());
        for (x, character) in new_line.chars().enumerate() {
            let t_cell = TCell::new(
                Occupant::new(self.id, self.id_counter.next()),
                Glyph::new(fg_clr, None, character),
                Some(Position::new(self.pos.x + x as u16, self.pos.y + y as u16)),
                255,
            );
            t_cells.push(t_cell);
            self.state
                .upsert_change(StateChange::Create { new_t_cell: t_cell });
        }
        t_cells
    }

    fn update_line_by_char(
        &mut self,
        y: usize,
        mut old_t_cells: Vec<TCell>,
        new_line: &str,
        fg_clr: Option<Color>,
    ) -> Vec<TCell> {
        let old_len = old_t_cells.len();
        let new_chars: Vec<char> = new_line.chars().collect();
        let new_len = new_chars.len();
        let min_len = old_len.min(new_len);

        for i in 0..min_len {
            let t_cell = &mut old_t_cells[i];
            let new_char = new_chars[i];
            if t_cell.style.symbol != new_char {
                t_cell.style.symbol = new_char;
                self.state.upsert_change(StateChange::Update {
                    t_cell: *t_cell,
                    init_pos: t_cell.pos,
                });
            }
        }

        if new_len > old_len {
            for i in old_len..new_len {
                let new_char = new_chars[i];
                let t_cell = TCell::new(
                    Occupant::new(self.id, self.id_counter.next()),
                    Glyph::new(fg_clr, None, new_char),
                    Some(Position::new(self.pos.x + i as u16, self.pos.y + y as u16)),
                    255,
                );
                old_t_cells.push(t_cell);
                self.state
                    .upsert_change(StateChange::Create { new_t_cell: t_cell });
            }
        } else if new_len < old_len {
            for t_cell in old_t_cells.drain(new_len..) {
                self.state.upsert_change(StateChange::Delete {
                    occupant: t_cell.occ,
                    init_pos: t_cell.pos,
                });
            }
        }

        old_t_cells
    }

    fn get_t_cells(&self) -> Box<dyn Iterator<Item = &TCell> + '_> {
        Box::new(self.t_cells_per_line.iter().flat_map(|line| line.iter()))
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
