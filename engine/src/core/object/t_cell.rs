use crate::core::{global::{Position}, object::Occupant};
use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TCell {
    pub occ: Occupant,
    pub style: Glyph,
    pub pos: Position,
}

impl TCell {
    pub fn new(occ: Occupant, style: Glyph, pos: Option<Position>) -> Self {
        TCell {
            occ,
            style,
            pos: {
                match pos {
                    Some(pos) => pos,
                    None => pos.unwrap_or_default(),
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

impl Glyph {
    pub fn new(fg_clr: Option<Color>, bg_clr: Option<Color>, symbol: char) -> Self {
        Self {
            fg_clr,
            bg_clr,
            symbol,
        }
    }
}
