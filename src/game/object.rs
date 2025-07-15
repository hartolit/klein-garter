use std::collections::HashMap;
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

pub trait Object {
    fn id(&self) -> Id;
    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_>;
    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_>;
}

///
/// COLLISION AND STATECHANGE
///
pub struct Collision<'a> {
    pub pos: Position,
    pub kind: &'a CellKind,
    pub colliders: &'a [ObjectRef],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateChange {
    Update {
        obj_id: Id,
        element_id: Id,
        element: Element,
        init_pos: Position,
    },
    Delete {
        obj_id: Id,
        element_id: Id,
        init_pos: Position,
    },
    Create {
        obj_id: Id,
        element_id: Id,
        new_element: Element,
    },
    Consume {
        obj_id: Id,
        element_id: Id,
        pos: Position,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateManager {
    pub changes: HashMap<(Id, Id), StateChange>,
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
                obj_id, element_id, ..
            } => (obj_id, element_id),
            StateChange::Update {
                obj_id, element_id, ..
            } => (obj_id, element_id),
            StateChange::Delete {
                obj_id, element_id, ..
            } => (obj_id, element_id),
            StateChange::Consume {
                obj_id, element_id, ..
            } => (obj_id, element_id),
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
                        init_pos: curr_old_pos,
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
                            obj_id, element_id, ..
                        } => {
                            *curr_state = StateChange::Delete {
                                obj_id,
                                element_id,
                                init_pos: *curr_old_pos,
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

pub trait DynamicObject: Object {
    fn next_pos(&self) -> Box<dyn Iterator<Item = Position> + '_>;
    fn update(
        &mut self,
        collisions: Option<Vec<Collision>>,
    ) -> Option<HashMap<(Id, Id), StateChange>>;
}

// pub trait InteractObject: Object {

// }
