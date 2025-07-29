use std::fmt::Debug;

pub mod element;
pub mod state;

use super::global::{Id, Position};
use super::grid::Collision;
use element::Element;
use state::{StateChange};

pub trait Object<'a>: Debug {
    fn id(&self) -> Id;
    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + 'a>;
    fn positions(&self) -> Box<dyn Iterator<Item = Position> + 'a>;
    fn state_changes(&self) -> Box<dyn Iterator<Item = &StateChange> + 'a>;

    // Behavior methods
    fn as_consumable(&self) -> Option<&dyn Consumable> {
        None
    }
    fn as_damaging(&self) -> Option<&dyn Damaging> {
        None
    }
    fn as_movable_mut(&mut self) -> Option<&mut (dyn Movable<'a> + 'a)> {
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

pub trait Movable<'a> {
    fn next_pos(&self) -> Box<dyn Iterator<Item = Position> + 'a>;
    fn update(
        &mut self,
        collisions: Box<dyn Iterator<Item = Collision<'a>> + 'a>,
    );
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
pub enum GameAction {
    RemoveObject { id: Id },
    //HitObject { owner: Id, hit: Occupant },
}

// ! Moved to enum dispatch based approach due to lifetime
// ! issues downcasting a non-'static type
// // Extended to keep object dynamic example
// //if let Some(object) = self.objects.get_mut(&snake_id) {
// //    if let Some(snake) = object.get_mut::<Snake>() {
// //        snake.direction = *direction;
// //    }
// //}
// pub trait ObjectExt {
//     fn get<T: 'static>(&self) -> Option<&T>;
//     fn get_mut<T: 'static>(&mut self) -> Option<&mut T>;
// }

// impl ObjectExt for dyn Object {
//     fn get<T: 'static>(&self) -> Option<&T> {
//         self.as_any().downcast_ref::<T>()
//     }

//     fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
//         self.as_any_mut().downcast_mut::<T>()
//     }
// }