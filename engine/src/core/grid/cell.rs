use crossterm::style::Color;

use crate::core::object::{element::Glyph, state::Occupant};

#[derive(Debug, Clone, Copy)]
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
}

impl Cell {
    pub fn new(kind: Kind) -> Self {
        Cell { occ_by: None, kind }
    }
}
