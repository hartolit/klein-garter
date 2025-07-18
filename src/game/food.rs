use std::iter;

use crate::game::object::{Consumable, Element, Glyph, Id, Object, Occupant, Position, StateChange};
use crossterm::style::Color;
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    Cherry,
    Mouse,
    Bomb,
    Grower,
}

// TODO - Change food to contain multiple elements (requires update loop and collision checks)
#[derive(Debug)]
pub struct Food {
    id: Id,
    kind: Kind,
    meal: i16,
    body: Element,
}

impl Food {
    pub fn new(obj_id: Id, kind: Kind, pos: Position) -> Self {
        let (meals, symbol, color) = match kind {
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
            Kind::Bomb => (
                -10,
                '⍟',
                Color::Rgb {
                    r: 169,
                    g: 169,
                    b: 169,
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
            meal: meals,
            body: Element {
                id: Id::new(0),
                style: Glyph {
                    fg_clr: Some(color),
                    bg_clr: None,
                    symbol,
                },
                pos,
            },
        }
    }

    pub fn replace_meal(&mut self, meals: i16) {
        self.meal = meals;
    }

    // TODO - Include new food type
    pub fn rng_food(obj_id: Id, pos: Position) -> Self {
        let food = match rand::rng().random_range(0..=2) {
            0 => Food::new(obj_id, Kind::Cherry, pos),
            1 => Food::new(obj_id, Kind::Mouse, pos),
            _ => Food::new(obj_id, Kind::Bomb, pos),
        };

        food
    }
}

impl Object for Food {
    fn id(&self) -> Id {
        self.id
    }

    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_> {
        Box::new(iter::once(&self.body))
    }

    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_> {
        Box::new(iter::once(self.body.pos))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_consumable(&self) -> Option<&dyn Consumable> {
        Some(self)
    }
}

impl Consumable for Food {
    fn get_meal(&self) -> i16 {
        self.meal
    }

    fn on_consumed(&self, hit_element_id: Id, pos: Position, _consumer_id: Id) -> StateChange {
        StateChange::Consume { occupant: Occupant::new(self.id, hit_element_id), pos }
    }
}
