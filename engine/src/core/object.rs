use std::any::Any;
use std::fmt::Debug;

pub mod element;
pub mod state;

use crate::core::object::state::StateManager;

use super::global::{Id, Position};
use super::grid::cell::CellRef;
use element::Element;
use state::StateChange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Occupant {
    pub obj_id: Id,
    pub element_id: Id,
}

impl Occupant {
    pub fn new(obj_id: Id, element_id: Id) -> Self {
        Occupant { obj_id, element_id }
    }
}

pub trait Object: Debug {
    fn id(&self) -> Id;
    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_stateful(&self) -> Option<&dyn Stateful> {
        None
    }
    fn as_stateful_mut(&mut self) -> Option<&mut dyn Stateful>{
        None
    }
    fn as_movable(&self) -> Option<&dyn Movable> {
        None
    }
    fn as_movable_mut(&mut self) -> Option<&mut dyn Movable> {
        None
    }
    fn as_destructible(&self) -> Option<&dyn Destructible> {
        None
    }
    fn as_destructible_mut(&mut self) -> Option<&mut dyn Destructible> {
        None
    }
}

pub trait Movable {
    fn predict_pos(&self) -> Box<dyn Iterator<Item = Position> + '_>;
    fn make_move(&mut self, probe: Vec<CellRef>) -> Vec<Action>;
}

pub trait Stateful {
    fn state_manager_mut(&mut self) -> &mut StateManager;
    fn state_manager(&self) -> &StateManager;
    fn state_changes(&self) -> Box<dyn Iterator<Item = &StateChange> + '_>;
}

pub trait Destructible: Object + Stateful {
    fn kill(&mut self) {
        let id = self.id();
        let elements_data: Vec<_> = self.elements().map(|e| (e.id, e.pos)).collect();
        let state_manager = self.state_manager_mut();

        for (element_id, pos) in elements_data {
            state_manager.upsert_change(StateChange::Delete { occupant: Occupant::new(id, element_id), init_pos: pos });
        }
    }

    fn is_dead(&self) -> bool;
}

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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Action {
    Collision { owner: Occupant, target: Occupant },
    Kill { obj_id: Id },
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
