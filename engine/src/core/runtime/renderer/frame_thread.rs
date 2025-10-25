use std::{io::{stdout, Stdout, Write}, sync::mpsc::Receiver};
use crossterm::{QueueableCommand, cursor, execute, style, terminal};

use crate::{core::runtime::renderer::RenderCommand, prelude::{Glyph, Position}};

pub enum Operation {
    Clear,
    Draw { glyph: Glyph, z_index: u8 },
}

// TODO - Take a snapshot from the buffer to sync to an infinite grid and reflect changes based on previous changes
// Right now the Operations outside of a grid is only reflected in its current StateChanges for the tick.
// That is to say previous states with a higher index than the current StateChanges might get overriden
// due to previous states not being reflected into its current states.
pub struct FrameThread {
    stdout: Stdout,
}

impl FrameThread {
    pub fn new() -> Self {
        let mut stdout = stdout();

        terminal::enable_raw_mode().unwrap();
        execute!(stdout, cursor::Hide).unwrap();

        Self {
            stdout,
        }
    }

    pub fn run(&mut self, rx: Receiver<RenderCommand>) {
        for command in rx {
            match command {
                RenderCommand::Draw(frame_buffer, is_full_render) => {
                    if is_full_render {
                        self.stdout
                            .queue(terminal::Clear(terminal::ClearType::All))
                            .unwrap();
                    }

                    for (pos, operation) in frame_buffer {
                        match operation {
                            Operation::Clear => Self::clear_glyph(&mut self.stdout, pos),
                            Operation::Draw { glyph, .. } => Self::draw_glyph(&mut self.stdout, glyph, pos),
                        }
                    }
                    self.stdout.flush().unwrap();
                },
                RenderCommand::Kill => {
                    terminal::disable_raw_mode().unwrap();
                    execute!(self.stdout, cursor::Show).unwrap();
                }
            }
        }
    }

    fn draw_glyph(stdout: &mut Stdout, glyph: Glyph, pos: Position) {
        if let Some(fg_color) = glyph.fg_clr {
            stdout.queue(style::SetForegroundColor(fg_color)).unwrap();
        } else {
            stdout
                .queue(style::SetForegroundColor(style::Color::Reset))
                .unwrap();
        }

        if let Some(bg_color) = glyph.bg_clr {
            stdout.queue(style::SetBackgroundColor(bg_color)).unwrap();
        } else {
            stdout
                .queue(style::SetBackgroundColor(style::Color::Reset))
                .unwrap();
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
