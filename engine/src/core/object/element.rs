use crate::core::global::{Id, Position};
use crossterm::style::Color;

///
/// ELEMENT
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Element {
    pub id: Id,
    pub style: Glyph,
    pub pos: Position,
}

impl Element {
    pub fn new(id: Id, style: Glyph, pos: Option<Position>) -> Self {
        Element {
            id,
            style,
            pos: {
                match pos {
                    Some(pos) => pos,
                    None => Position { x: 0, y: 0 },
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Glyph {
    pub fg_clr: Option<Color>,
    pub bg_clr: Option<Color>,
    pub symbol: char,
}
