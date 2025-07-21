use crossterm::style::Color;

use crate::core::{
    global::Position,
    object::{element::Glyph, state::Occupant},
};

#[derive(Debug, Clone, Copy)]
pub enum CellKind {
    Ground,
    Water,
    Lava,
    Border,
}

impl CellKind {
    pub fn appearance(&self) -> Glyph {
        match self {
            CellKind::Ground => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::Black),
                symbol: ' ',
            },
            CellKind::Water => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkBlue),
                symbol: '≈',
            },
            CellKind::Lava => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkRed),
                symbol: '#',
            },
            CellKind::Border => Glyph {
                bg_clr: Option::Some(Color::Black),
                fg_clr: Option::Some(Color::DarkGrey),
                symbol: '█',
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridCell {
    pub occ_by: Option<Occupant>,
    pub kind: CellKind,
}

impl GridCell {
    pub fn new(kind: CellKind) -> Self {
        GridCell { occ_by: None, kind }
    }
}

///
/// COLLISION
///
pub struct Collision<'a> {
    pub pos: Position,
    pub kind: &'a CellKind,
    pub collider: Occupant,
}
