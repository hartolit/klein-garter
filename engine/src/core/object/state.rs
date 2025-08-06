use std::collections::{HashMap, hash_map::Entry};
use std::mem;

use super::Occupant;
use super::element::Element;
use crate::core::global::Position;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub changes: HashMap<Occupant, StateChange>,
}

impl State {
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    pub fn upsert_change(&mut self, new_state: StateChange) {
        let key = match new_state {
            StateChange::Create { occupant, .. } => occupant,
            StateChange::Update { occupant, .. } => occupant,
            StateChange::Delete { occupant, .. } => occupant,
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
                        StateChange::Delete { occupant, .. } => {
                            *curr_state = StateChange::Delete {
                                occupant,
                                init_pos: *curr_init_pos,
                            };
                        }
                    },
                    StateChange::Delete { .. } => {}
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(new_state);
            }
        }
    }

    pub fn drain_changes(&mut self) -> HashMap<Occupant, StateChange> {
        mem::take(&mut self.changes)
    }
}