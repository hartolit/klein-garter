use std::iter;

use crossterm::style::Color;
use crate::game::object::{Element, Glyph, Object, ObjectId, Position};
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub enum FoodKind {
    Cherry,
    Mouse,
    Bomb,
}

#[derive(Debug)]
pub struct Food {
    pub id: ObjectId,
    pub kind: FoodKind,
    pub meals: i16,
    pub body: Element,
}

impl Food {
    pub fn new(obj_id: ObjectId, kind: FoodKind, pos: Position) -> Self {
        let (meals, symbol, color) = match kind {
            FoodKind::Cherry => (1, '🍒', Color::Rgb { r: 255, g: 0, b: 0 }),
            FoodKind::Mouse => (2, '🐁', Color::Rgb { r: 50, g: 60, b: 70 }),
            FoodKind::Bomb => (-10, '💣', Color::Rgb { r: 0, g: 0, b: 0 }),
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

    pub fn rng_food(obj_id: ObjectId, pos: Position) -> Self {
        let food = match rand::rng().random_range(0..=2) {
            0 => Food::new(obj_id, FoodKind::Cherry, pos),
            1 => Food::new(obj_id, FoodKind::Mouse, pos),
            _ => Food::new(obj_id, FoodKind::Bomb, pos),
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