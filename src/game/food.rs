use std::iter;

use crossterm::style::Color;
use crate::game::object::{Element, Glyph, Object, ObjectId, Position};
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    Cherry,
    Mouse,
    Bomb,
}

#[derive(Debug)]
pub struct Food {
    id: ObjectId,
    kind: Kind,
    meals: i16,
    body: Element,
}

impl Food {
    pub fn new(obj_id: ObjectId, kind: Kind, pos: Position) -> Self {
        let (meals, symbol, color) = match kind {
            Kind::Cherry => (1, '🍒', Color::Rgb { r: 255, g: 0, b: 0 }),
            Kind::Mouse => (2, '🐁', Color::Rgb { r: 50, g: 60, b: 70 }),
            Kind::Bomb => (-10, '💣', Color::Rgb { r: 0, g: 0, b: 0 }),
        };

        Self {
            id: obj_id,
            kind,
            meals,
            body: Element { 
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

    pub fn rng_food(obj_id: ObjectId, pos: Position) -> Self {
        let food = match rand::rng().random_range(0..=2) {
            0 => Food::new(obj_id, Kind::Cherry, pos),
            1 => Food::new(obj_id, Kind::Mouse, pos),
            _ => Food::new(obj_id, Kind::Bomb, pos),
        };

        food
    }
}

impl Object for Food {
    fn id(&self) -> ObjectId {
        self.id
    }

    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_> {
        Box::new(iter::once(&self.body))
    }

    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_> {
        Box::new(iter::once(self.body.pos))
    }
}