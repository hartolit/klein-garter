use super::utils::Position;
use std::io::{self, Stdout, Write};
use std::fmt::Display;

#[derive(Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

#[derive(Debug)]
pub struct Renderer {
    out: Stdout,
}

// Renderer based on ANSI Escape Sequences
// ref: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
// TODO - handle unwrap();
impl Renderer {
    // Create new renderer
    pub fn new() -> Self {
        Self { out: io::stdout() }
    }

    // Set foreground color
    // ESC[38;2;{r};{g};{b}m
    pub fn set_fg(&mut self, rgb: &Rgb) {
        write!(self.out, "\x1B[38;2;{};{};{}m", rgb.0, rgb.1, rgb.2).unwrap();
    }

    // Set background color
    // ESC[48;2;{r};{g};{b}m
    pub fn set_bg(&mut self, rgb: &Rgb) {
        write!(self.out, "\x1B[48;2;{};{};{}m", rgb.0, rgb.1, rgb.2).unwrap();
    }

    pub fn set_cursor(&mut self, pos: &Position) {
        write!(self.out, "\x1B[{};{}H", pos.y + 1, pos.x + 1).unwrap();
    }

    pub fn write<T: Display>(&mut self, value: T) {
        write!(self.out, "{}", value).unwrap();
    } 

    // Clear terminal
    pub fn clear_styles(&mut self) {
        write!(self.out, "\x1B[0m").unwrap();
    }

    pub fn clear_screen(&mut self) {
        write!(self.out, "\x1B[2J\x1B[H").unwrap();
        self.out.flush().unwrap();
    }

    // Flush buffer
    pub fn flush(&mut self) {
        self.out.flush().unwrap();
    }
}