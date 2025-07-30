use std::any::Any;
use std::fmt::Debug;

pub mod element;
pub mod state;

use super::global::{Id, Position};
use super::grid::Collision;
use element::Element;
use state::{StateChange};

pub trait Object: Debug {
    fn id(&self) -> Id;
    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_>;
    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_>;
    fn state_changes(&self) -> Box<dyn Iterator<Item = &StateChange> + '_>;

    // Methods for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    // Behavior methods
    fn interact(&mut self) {}
    fn damage(&mut self) {}
    fn kill(&mut self) {}

    fn as_consumable(&self) -> Option<&dyn Consumable> {
        None
    }
    fn as_damaging(&self) -> Option<&dyn Damaging> {
        None
    }
    fn as_movable(&self) -> Option<&dyn Movable> {
        None
    }
    fn as_movable_mut(&mut self) -> Option<&mut dyn Movable> {
        None
    }
}

pub trait Consumable {
    fn get_meal(&self) -> i16;
    fn on_consumed(&self, hit_element_id: Id, pos: Position, recipient_id: Id) -> StateChange;
}

pub trait Damaging {
    fn get_damage(&self) -> i16;
    fn on_hit(&self, hit_element_id: Id, pos: Position, recipient_id: Id) -> StateChange;
}

pub trait Movable {
    fn next_pos(&self) -> Box<dyn Iterator<Item = Position> + '_>;
    fn update(&mut self, collisions: Vec<Collision>) -> Vec<Action>;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BodySegment {
    pub orientation: Orientation,
    pub elements: Vec<Element>,
}

impl BodySegment {
    pub fn new(orientation: Orientation, elements: Vec<Element>) -> Self {
        Self {
            orientation,
            elements,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Action {
    Interact { owner: Id, object: Id },
    Damage { owner: Id, object: Id, damage: u16 },
    Kill { owner: Id, kill: Id },
}


// Extended for dyn object:
// Use example:
// if let Some(object) = self.objects.get_mut(&snake_id) {
//     if let Some(snake) = object.get_mut::<Snake>() {
//         snake.direction = *direction;
//     }
// }
pub trait ObjectExt {
    fn get<T: 'static>(&self) -> Option<&T>;
    fn get_mut<T: 'static>(&mut self) -> Option<&mut T>;
}

impl ObjectExt for dyn Object {
    fn get<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}
