use std::iter;

use crossterm::style::Color;
use crate::game::object::{Element, Glyph, Object, Id, Position};
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    Cherry,
    Mouse,
    Bomb,
    Grower,
}

#[derive(Debug)]
pub struct Food {
    id: Id,
    kind: Kind,
    meals: i16,
    body: Element,
}

impl Food {
    pub fn new(obj_id: Id, kind: Kind, pos: Position) -> Self {
        let (meals, symbol, color) = match kind {
            Kind::Cherry => (1, '⧝', Color::Rgb { r: 169, g: 42, b: 69 }), 
            Kind::Mouse => (2, '⦺', Color::Rgb { r: 42, g: 69, b: 69 }),
            Kind::Bomb => (-10, '⍟', Color::Rgb { r: 169, g: 169, b: 169 }),
            Kind::Grower => (0, '⌘', Color::Rgb { r: 242, g: 242, b: 69 })
        };

        Self {
            id: obj_id,
            kind,
            meals,
            body: Element { 
                id: Id::new(0),
                style: Glyph { 
                    fg_clr: Some(color),
                    bg_clr: None,
                    symbol
                }, 
                pos 
            }
        }
    }

    pub fn replace_meal(&mut self, meals: i16) {
        self.meals = meals;
    }

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
}