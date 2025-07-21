use std::fmt::Debug;
use std::{any::Any, collections::HashMap};

pub mod element;
pub mod state;

use super::global::{Id, Position};
use super::grid::cell::Collision;
use element::Element;
use state::{Occupant, StateChange};

///
/// OBJECT
///
pub trait Object: Any + Debug {
    fn id(&self) -> Id;
    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_>;
    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_>;

    // Methods for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Behavior methods
    fn as_consumable(&self) -> Option<&dyn Consumable> {
        None
    }
    fn as_damaging(&self) -> Option<&dyn Damaging> {
        None
    }
    fn as_movable(&self) -> Option<&dyn Movable> {
        None
    }
}

// Extended to keep object dynamic
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

pub trait Consumable {
    fn get_meal(&self) -> i16;
    fn on_consumed(&self, hit_element_id: Id, pos: Position, recipient_id: Id) -> StateChange;
}

pub trait Damaging {
    fn get_damage(&self) -> i16;
    fn on_hit(&self, hit_element_id: Id, pos: Position, recipient_id: Id) -> StateChange;
}

// TODO - add event/message queue system (future update)
pub trait Movable {
    fn next_pos(&self) -> Box<dyn Iterator<Item = Position> + '_>;
    fn update(
        &mut self,
        collisions: Option<Vec<Collision>>,
        game_objects: &HashMap<Id, Box<dyn Object>>,
    ) -> Option<HashMap<Occupant, StateChange>>;
}

///
/// BodySegment
///
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
