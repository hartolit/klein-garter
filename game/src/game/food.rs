use std::iter;

use ::engine::core::{
    global::{Id, Position},
    object::{
        Destructible, Object, Occupant, Stateful,
        t_cell::{Glyph, TCell},
    },
};
use engine::core::object::state::State;

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
    state: State,
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
            },
            state: State::new(),
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

    fn t_cells(&self) -> Box<dyn Iterator<Item = &TCell> + '_> {
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
        &self.state
    }

    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

impl Destructible for Food {}

impl Consumable for Food {
    fn get_meal(&self) -> i16 {
        self.meal
    }
}
