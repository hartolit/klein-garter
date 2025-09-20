use crossterm::style::Color;
use rand::Rng;

use ::engine::prelude::*;

use super::Damaging;

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    LittleBoy,
    FatMan,
    ThinMan,
}

#[derive(Debug)]
pub struct Bomb {
    id: Id,
    kind: Kind,
    damage: u16,
    body: TCell,
}

impl Bomb {
    pub fn new(obj_id: Id, kind: Kind, pos: Position) -> Self {
        let (damage, symbol, color) = match kind {
            Kind::LittleBoy => (
                2,
                '⏺',
                Color::Rgb {
                    r: 169,
                    g: 169,
                    b: 169,
                },
            ),
            Kind::FatMan => (
                4,
                '᳀',
                Color::Rgb {
                    r: 169,
                    g: 169,
                    b: 169,
                },
            ),
            Kind::ThinMan => (
                6,
                '۩',
                Color::Rgb {
                    r: 169,
                    g: 169,
                    b: 169,
                },
            ),
        };

        let glyph = Glyph {
            fg_clr: Some(color),
            bg_clr: None,
            symbol,
        };

        Self {
            id: obj_id,
            kind,
            damage,
            body: TCell::new(Occupant::new(obj_id, Id::new(0)), glyph, Some(pos), 0),
        }
    }

    pub fn rng_bomb(obj_id: Id, pos: Position) -> Self {
        let bomb = match rand::rng().random_range(0..=2) {
            0 => Bomb::new(obj_id, Kind::LittleBoy, pos),
            1 => Bomb::new(obj_id, Kind::FatMan, pos),
            _ => Bomb::new(obj_id, Kind::ThinMan, pos),
        };

        bomb
    }
}

define_object! {
    struct Bomb,
    id_field: id,
    t_cells: single(body),
    capabilities: {
        Spatial {}
        Destructible {}
    }
}

impl Damaging for Bomb {
    fn get_damage(&self) -> u16 {
        self.damage
    }
}
