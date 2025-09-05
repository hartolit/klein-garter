use std::{collections::HashMap, io::{stdout, Stdout, Write}};

use crossterm::{cursor, execute, style, terminal, QueueableCommand};

use crate::prelude::{Glyph, Position};

pub enum Operation {
    Clear,
    Draw { glyph: Glyph, z_index: u8 }
}

pub struct Buffer {
    stdout: Stdout,
    frame_buffer: HashMap<Position, Operation>,
}

impl Buffer {
    pub fn new() -> Self {
        let mut stdout = stdout();

        terminal::enable_raw_mode().unwrap();
        execute!(stdout, cursor::Hide).unwrap();

        Self {
            stdout,
            frame_buffer: HashMap::new(),
        }
    }

    pub fn upsert(&mut self, pos: Position, new_op: Operation) {
        use std::collections::hash_map::Entry;

        let new_z = match &new_op {
            Operation::Clear => 0,
            Operation::Draw { z_index, .. } => *z_index,
        };

        match self.frame_buffer.entry(pos) {
            Entry::Vacant(entry) => {
                entry.insert(new_op);
            }
            Entry::Occupied(mut entry) => {
                let existing_op = entry.get_mut();
                let existing_z = match existing_op {
                    Operation::Clear => 0,
                    Operation::Draw { z_index, .. } => *z_index,
                };

                if new_z > existing_z {
                    *existing_op = new_op;
                }
            }
        }
    }

    pub fn kill(&mut self) {
        terminal::disable_raw_mode().unwrap();
        execute!(self.stdout, cursor::Show).unwrap();
    }

    pub fn clear(&mut self) {
        self.frame_buffer.clear();
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
    }

    pub fn flush(&mut self) {
        for (pos, operation) in self.frame_buffer.drain() {
            match operation {
                Operation::Clear => Self::clear_glyph(&mut self.stdout, pos),
                Operation::Draw { glyph, .. } => Self::draw_glyph(&mut self.stdout, glyph, pos),
            };
        }
        self.stdout.flush().unwrap();
    }

    pub fn draw_glyph(stdout: &mut Stdout, glyph: Glyph, pos: Position) {
        if let Some(fg_color) = glyph.fg_clr {
            stdout.queue(style::SetForegroundColor(fg_color)).unwrap();
        } else {
            stdout.queue(style::SetForegroundColor(style::Color::Reset)).unwrap();
        }

        if let Some(bg_color) = glyph.bg_clr {
            stdout.queue(style::SetBackgroundColor(bg_color)).unwrap();
        } else {
            stdout.queue(style::SetBackgroundColor(style::Color::Reset)).unwrap();
        }

        stdout
            .queue(cursor::MoveTo(pos.x, pos.y))
            .unwrap()
            .queue(style::Print(glyph.symbol))
            .unwrap();
    }

    fn clear_glyph(stdout: &mut Stdout, pos: Position) {
        stdout
            .queue(cursor::MoveTo(pos.x, pos.y))
            .unwrap()
            .queue(style::ResetColor)
            .unwrap()
            .queue(style::Print(' '))
            .unwrap();
    }
}