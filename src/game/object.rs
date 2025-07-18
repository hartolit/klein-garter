use std::fmt::Debug;
use std::{any::Any, collections::HashMap};
use std::collections::hash_map::Entry;

use super::grid::{CellKind, ObjectRef};
use crossterm::style::Color;

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

///
/// RESIZESTATE
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeState {
    Normal { size: usize },
    Brief { size: usize, native_size: usize },
}

impl ResizeState {
    pub fn size(&self) -> usize {
        match self {
            ResizeState::Normal { size } => *size,
            ResizeState::Brief { size, .. } => *size,
        }
    }

    pub fn native(&self) -> usize {
        match self {
            ResizeState::Normal { size } => *size,
            ResizeState::Brief { native_size, .. } => *native_size,
        }
    }
}

///
/// ID AND ID GENERATION
/// TODO - Change Id to explicit types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id {
    pub value: u64,
}

impl Id {
    pub fn new(id: u64) -> Self {
        Id { value: id }
    }
}

#[derive(Debug, Clone)]
pub struct IdCounter {
    counter: Id,
}

impl IdCounter {
    pub fn new() -> Self {
        Self {
            counter: Id::new(0),
        }
    }

    pub fn next(&mut self) -> Id {
        let id = self.counter.value;
        self.counter.value += 1;
        Id::new(id)
    }
}

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

///
/// POSITION
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

///
/// ELEMENT AND OBJECT TRAIT
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Glyph {
    pub fg_clr: Option<Color>,
    pub bg_clr: Option<Color>,
    pub symbol: char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Element {
    pub id: Id,
    pub style: Glyph,
    pub pos: Position,
}

impl Element {
    pub fn new(id: Id, style: Glyph, pos: Option<Position>) -> Self {
        Element {
            id,
            style,
            pos: {
                match pos {
                    Some(pos) => pos,
                    None => Position { x: 0, y: 0 },
                }
            },
        }
    }
}

pub trait Consumable {
    fn get_meal(&self) -> i16;
    fn on_consumed(&self, consumer_id: Id) -> StateChange;
}

pub trait Damaging {
    fn get_damage(&self) -> i32;
}

// TODO - Add event/message queue and handler
pub trait Movable {
    fn next_pos(&self) -> Box<dyn Iterator<Item = Position> + '_>;
    fn update(
        &mut self,
        collisions: Option<Vec<Collision>>,
    ) -> Option<HashMap<(Id, Id), StateChange>>;
}

pub trait Object: Any + Debug {
    fn id(&self) -> Id;
    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_>;
    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_>;

    // Methods for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Behavior methods
    fn as_consumable(&self) -> Option<&dyn Consumable> { None }
    fn as_damaging(&self) -> Option<&dyn Damaging> { None }
    fn as_movable(&self) -> Option<&dyn Movable> { None }
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


///
/// COLLISION AND STATECHANGE
///
pub struct Collision<'a> {
    pub pos: Position,
    pub kind: &'a CellKind,
    pub collider: Occupant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateChange {
    Update {
        occupant: Occupant,
        element: Element,
        init_pos: Position,
    },
    Delete {
        occupant: Occupant,
        init_pos: Position,
    },
    Create {
        occupant: Occupant,
        new_element: Element,
    },
    Consume {
        occupant: Occupant,
        pos: Position,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateManager {
    pub changes: HashMap<Occupant, StateChange>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    pub fn upsert_change(&mut self, new_state: StateChange) {
        let key = match new_state {
            StateChange::Create {
                occupant, ..
            } => occupant,
            StateChange::Update {
                occupant, ..
            } => occupant,
            StateChange::Delete {
                occupant, ..
            } => occupant,
            StateChange::Consume {
                occupant, ..
            } => occupant,
        };

        match self.changes.entry(key) {
            Entry::Occupied(mut entry) => {
                let curr_state = entry.get_mut();

                match curr_state {
                    StateChange::Create {
                        new_element: curr_element,
                        ..
                    } => match new_state {
                        StateChange::Create { new_element, .. } => {
                            *curr_element = new_element;
                        }
                        StateChange::Update { element, .. } => {
                            *curr_element = element;
                        }
                        StateChange::Consume { .. } => {}
                        StateChange::Delete { .. } => {
                            entry.remove();
                        }
                    },

                    StateChange::Update {
                        element: curr_element,
                        init_pos: curr_init_pos,
                        ..
                    } => match new_state {
                        StateChange::Create { new_element, .. } => {
                            *curr_element = new_element;
                        }
                        StateChange::Update { element, .. } => {
                            *curr_element = element;
                        }
                        StateChange::Consume { .. } => {}
                        StateChange::Delete {
                            occupant, ..
                        } => {
                            *curr_state = StateChange::Delete {
                                occupant,
                                init_pos: *curr_init_pos,
                            };
                        }
                    },

                    StateChange::Consume { .. } => {}
                    StateChange::Delete { .. } => {}
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(new_state);
            }
        }
    }
}

// pub trait InteractObject: Object {

// }
