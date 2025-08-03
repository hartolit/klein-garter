use crossterm::style::Color;
use engine::core::object::state::StateManager;
use rand::Rng;
use std::iter;

use ::engine::core::{
    global::{Id, Position},
    object::{
        Object, Occupant, Stateful, Destructible,
        element::{Element, Glyph},
        state::StateChange,
    },
};

use super::object_ext::Damaging;

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
    damage: i16,
    body: Element,
    state_manager: StateManager,
    is_dead: bool,
    //pub effect_area: u16,
}

impl Bomb {
    pub fn new(obj_id: Id, kind: Kind, pos: Position) -> Self {
        let (damage, symbol, color) = match kind {
            Kind::LittleBoy => (
                -2,
                '⏺',
                Color::Rgb {
                    r: 169,
                    g: 169,
                    b: 169,
                },
            ),
            Kind::FatMan => (
                -10,
                '᳀',
                Color::Rgb {
                    r: 169,
                    g: 169,
                    b: 169,
                },
            ),
            Kind::ThinMan => (
                -5,
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
            body: Element::new(Id::new(0), glyph, Some(pos)),
            state_manager: StateManager::new(),
            is_dead: false,
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

impl Object for Bomb {
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

impl Stateful for Bomb {
    fn state_manager(&self) -> &StateManager {
        &self.state_manager
    }

    fn state_manager_mut(&mut self) -> &mut StateManager {
        &mut self.state_manager
    }

    fn state_changes(&self) -> Box<dyn Iterator<Item = &StateChange> + '_> {
        Box::new(self.state_manager.changes.values())
    }
}

impl Destructible for Bomb {}

impl Damaging for Bomb {
    fn get_damage(&self) -> i16 {
        self.damage
    }
}
