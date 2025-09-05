use std::collections::{HashMap, hash_map::Entry};
use std::mem;

use super::Occupant;
use super::t_cell::TCell;
use crate::core::global::Position;

/// StateChange enum is used for determining
/// a cells state. 
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateChange {
    Update {
        t_cell: TCell,
        init_pos: Position,
    },
    Delete {
        occupant: Occupant,
        init_pos: Position,
    },
    Create {
        new_t_cell: TCell,
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
            StateChange::Create { new_t_cell, .. } => new_t_cell.occ,
            StateChange::Update { t_cell, .. } => t_cell.occ,
            StateChange::Delete { occupant, .. } => occupant,
        };

        match self.changes.entry(key) {
            Entry::Occupied(mut entry) => {
                let curr_state = entry.get_mut();

                match curr_state {
                    StateChange::Create {
                        new_t_cell: curr_t_cell,
                        ..
                    } => match new_state {
                        StateChange::Create { new_t_cell, .. } => {
                            *curr_t_cell = new_t_cell;
                        }
                        StateChange::Update { t_cell, .. } => {
                            *curr_t_cell = t_cell;
                        }
                        StateChange::Delete { .. } => {
                            entry.remove();
                        }
                    },

                    StateChange::Update {
                        t_cell: curr_t_cell,
                        init_pos: curr_init_pos,
                        ..
                    } => match new_state {
                        StateChange::Create { new_t_cell, .. } => {
                            *curr_t_cell = new_t_cell;
                        }
                        StateChange::Update { t_cell, .. } => {
                            *curr_t_cell = t_cell;
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
