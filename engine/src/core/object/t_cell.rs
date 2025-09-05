use crate::core::{global::Position, object::Occupant};
use crossterm::style::Color;

/// TCell (Terminal Cell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TCell {
    pub occ: Occupant,
    pub style: Glyph,
    pub pos: Position,
    pub z_index: u8, // TODO - Add to Position struct
}

impl TCell {
    pub fn new(occ: Occupant, style: Glyph, pos: Option<Position>, z_index: u8) -> Self {
        TCell {
            occ,
            style,
            pos: {
                match pos {
                    Some(pos) => pos,
                    None => pos.unwrap_or_default(),
                }
            },
            z_index,
        }
    }
}

/// Glyph represents graphical data of a cell
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
