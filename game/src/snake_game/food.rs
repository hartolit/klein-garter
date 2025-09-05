use ::engine::core::{
    global::{Id, Position},
    object::{
        Occupant,
        t_cell::{Glyph, TCell},
    },
};
use engine::{define_object};

use crossterm::style::Color;
use rand::Rng;

use super::game_object::Consumable;

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    Cherry,
    Mouse,
    Grower,
}

// TODO - Change food to contain multiple elements (requires update loop and collision checks)
#[derive(Debug)]
pub struct Food {
    id: Id,
    kind: Kind,
    meal: i16,
    body: TCell,
}

impl Food {
    pub fn new(obj_id: Id, kind: Kind, pos: Position) -> Self {
        let (meal, symbol, color) = match kind {
            Kind::Cherry => (
                1,
                '⧝',
                Color::Rgb {
                    r: 169,
                    g: 42,
                    b: 69,
                },
            ),
            Kind::Mouse => (
                2,
                '⦺',
                Color::Rgb {
                    r: 42,
                    g: 69,
                    b: 69,
                },
            ),
            Kind::Grower => (
                0,
                '⌘',
                Color::Rgb {
                    r: 242,
                    g: 242,
                    b: 69,
                },
            ),
        };

        Self {
            id: obj_id,
            kind,
            meal,
            body: TCell {
                occ: Occupant::new(obj_id, Id::new(0)),
                style: Glyph {
                    fg_clr: Some(color),
                    bg_clr: None,
                    symbol,
                },
                pos,
                z_index: 50,
            },
        }
    }

    pub fn rng_food(obj_id: Id, pos: Position) -> Self {
        let food = match rand::rng().random_range(0..=2) {
            0 => Food::new(obj_id, Kind::Cherry, pos),
            1 => Food::new(obj_id, Kind::Mouse, pos),
            _ => Food::new(obj_id, Kind::Grower, pos),
        };

        food
    }
}

define_object! {
    struct Food,
    id_field: id,
    t_cells: single(body),
    capabilities: {
    }
}

impl Consumable for Food {
    fn get_meal(&self) -> i16 {
        self.meal
    }
}
