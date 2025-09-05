use crossterm::style::Color;

use crate::core::global::Position;
use crate::core::object::Occupant;
use crate::core::object::t_cell::Glyph;

// TODO - Make this configurable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Ground,
    Water,
    Lava,
    Border,
}

impl Kind {
    pub fn appearance(&self) -> Glyph {
        match self {
            Kind::Ground => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::Black),
                symbol: ' ',
            },
            Kind::Water => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkBlue),
                symbol: '≈',
            },
            Kind::Lava => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkRed),
                symbol: '#',
            },
            Kind::Border => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkGrey),
                symbol: '█',
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub occ_by: Option<Occupant>,
    pub kind: Kind,
    pub z_index: u8
}

impl Cell {
    pub fn new(kind: Kind) -> Self {
        Cell { occ_by: None, kind, z_index: 0 }
    }
}

pub struct CellRef<'a> {
    pub pos: Position,
    pub cell: &'a Cell,
}

impl<'a> CellRef<'a> {
    pub fn new(pos: Position, cell: &'a Cell) -> Self {
        CellRef { pos, cell }
    }
}
