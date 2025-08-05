use std::iter;

use ::engine::core::{
    global::{Id, Position},
    object::{
        Object, Occupant, Stateful, Destructible,
        element::{Element, Glyph},
        state::StateChange,
    },
};
use engine::core::object::state::State;

use crossterm::style::Color;
use rand::Rng;

use super::object_ext::Consumable;

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
    body: Element,
    state_manager: State,
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
            body: Element {
                id: Id::new(0),
                style: Glyph {
                    fg_clr: Some(color),
                    bg_clr: None,
                    symbol,
                },
                pos,
            },
            state_manager: State::new(),
        }
    }

    pub fn replace_meal(&mut self, meals: i16) {
        self.meal = meals;
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

impl Object for Food {
    fn id(&self) -> Id {
        self.id
    }

    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_> {
        Box::new(iter::once(&self.body))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_stateful(&self) -> Option<&dyn Stateful> {
        Some(self)
    }
    
    fn as_stateful_mut(&mut self) -> Option<&mut dyn Stateful> {
        Some(self)
    }

    fn as_destructible(&self) -> Option<&dyn Destructible> {
        Some(self)
    }

    fn as_destructible_mut(&mut self) -> Option<&mut dyn Destructible> {
        Some(self)
    }
}

impl Stateful for Food {
    fn state(&self) -> &State {
        &self.state_manager
    }

    fn state_mut(&mut self) -> &mut State {
        &mut self.state_manager
    }

    fn state_changes(&self) -> Box<dyn Iterator<Item = &StateChange> + '_> {
        Box::new(self.state_manager.changes.values())
    }
}

impl Destructible for Food {}

impl Consumable for Food {
    fn get_meal(&self) -> i16 {
        self.meal
    }
}
