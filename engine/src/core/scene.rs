use std::collections::{HashMap, HashSet};

pub mod global_state;

use global_state::GlobalState;

use crate::core::event::Event;

use super::global::{Id, IdCounter};
use super::grid::SpatialGrid;
use super::object::{Object, state::StateChange};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum ObjectIndex {
    Movable,
    Destructible,
    Stateful,
}

pub struct Scene {
    pub id_counter: IdCounter,
    pub objects: HashMap<Id, Box<dyn Object>>,
    pub indexes: HashMap<ObjectIndex, HashSet<Id>>,
    pub spatial_grid: SpatialGrid,
    pub global_state: GlobalState,
    pub event_bus: Vec<Box<dyn Event>>,
}

impl Scene {
    pub fn new(spatial_grid: SpatialGrid) -> Self {
        Self {
            id_counter: IdCounter::new(),
            objects: HashMap::new(),
            indexes: HashMap::new(),
            spatial_grid,
            global_state: GlobalState::new(),
            event_bus: Vec::new(),
        }
    }

    pub fn attach_object<F>(&mut self, create_fn: F) -> Id
    where
        F: FnOnce(Id) -> Box<dyn Object>,
    {
        let new_id = self.id_counter.next();
        let new_object = create_fn(new_id);

        self.add_indexes(&new_object);
        self.spatial_grid.add_object(&new_object);

        // First draw
        for tcell in new_object.t_cells() {
            self.global_state.state.changes.insert(tcell.occ, StateChange::Create { new_t_cell: *tcell });
        }

        self.objects.insert(new_id, new_object);

        new_id
    }

    pub fn remove_object(&mut self, id: &Id) {
        if let Some(mut object) = self.objects.remove(id) {
            if let Some(destructable) = object.as_destructible_mut() {
                destructable.kill();
                self.global_state
                    .state
                    .changes
                    .extend(destructable.state_mut().drain_changes());
                self.remove_indexes(&object);
            }
        }
    }

    pub fn push_event<E: Event>(&mut self, event: E) {
        self.event_bus.push(Box::new(event));
    }

    pub fn sync(&mut self) {
        self.global_state.state.changes.clear();

        let stateful_ids: Vec<Id> = self
            .indexes
            .get(&ObjectIndex::Stateful)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default();

        for id in stateful_ids {
            if let Some(object) = self.objects.get_mut(&id) {
                if let Some(stateful) = object.as_stateful_mut() {
                    self.global_state
                        .state
                        .changes
                        .extend(stateful.state_mut().drain_changes());
                }
            }
        }

        self.global_state.finalize();

        for state in self.global_state.finalized.deletes.iter() {
            if let StateChange::Delete { occupant, init_pos } = state {
                self.spatial_grid.remove_cell_occ(*occupant, *init_pos);
            }
        }

        for state in self.global_state.finalized.updates.iter() {
            if let StateChange::Update { t_cell, init_pos } = state {
                if &t_cell.pos != init_pos {
                    self.spatial_grid.remove_cell_occ(t_cell.occ, *init_pos);
                    self.spatial_grid.add_cell_occ(t_cell.occ, t_cell.pos);
                }
            }
        }

        for state in self.global_state.finalized.creates.iter() {
            if let StateChange::Create { new_t_cell } = state {
                self.spatial_grid
                    .add_cell_occ(new_t_cell.occ, new_t_cell.pos);
            }
        }
    }

    fn add_indexes(&mut self, object: &Box<dyn Object>) {
        let id = object.id();

        if object.as_movable().is_some() {
            self.indexes
                .entry(ObjectIndex::Movable)
                .or_default()
                .insert(id);
        }

        if object.as_destructible().is_some() {
            self.indexes
                .entry(ObjectIndex::Destructible)
                .or_default()
                .insert(id);
        }

        if object.as_stateful().is_some() {
            self.indexes
                .entry(ObjectIndex::Stateful)
                .or_default()
                .insert(id);
        }
    }

    fn remove_indexes(&mut self, object: &Box<dyn Object>) {
        let id = object.id();

        if object.as_movable().is_some() {
            if let Some(set) = self.indexes.get_mut(&ObjectIndex::Movable) {
                set.remove(&id);
            }
        }

        if object.as_destructible().is_some() {
            if let Some(set) = self.indexes.get_mut(&ObjectIndex::Destructible) {
                set.remove(&id);
            }
        }

        if object.as_stateful().is_some() {
            if let Some(set) = self.indexes.get_mut(&ObjectIndex::Stateful) {
                set.remove(&id);
            }
        }
    }
}
