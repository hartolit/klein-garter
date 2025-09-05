use std::collections::{HashMap, HashSet};

pub mod global_state;

use global_state::GlobalState;

use crate::core::event::Event;

use super::global::{Id, IdCounter};
use super::grid::SpatialGrid;
use super::object::{Object, state::StateChange};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum ObjectIndex {
    Stateful,
    Destructible,
    Active,
    Spatial,
    Movable,
    StatefulSpatial,
}

pub struct Scene {
    pub id_counter: IdCounter,
    pub objects: HashMap<Id, Box<dyn Object>>,
    pub indexes: HashMap<ObjectIndex, HashSet<Id>>,
    pub spatial_grid: Option<SpatialGrid>,
    pub global_state: GlobalState,
    pub event_bus: Vec<Box<dyn Event>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            id_counter: IdCounter::new(),
            objects: HashMap::new(),
            indexes: HashMap::new(),
            spatial_grid: None,
            global_state: GlobalState::new(),
            event_bus: Vec::new(),
        }
    }

    pub fn attach_grid(&mut self, grid: SpatialGrid) {
        self.spatial_grid = Some(grid);
    }

    pub fn attach_object<F>(&mut self, create_fn: F) -> Id
    where
        F: FnOnce(Id) -> Box<dyn Object>,
    {
        let new_id = self.id_counter.next();
        let new_object = create_fn(new_id);

        self.add_indexes(&new_object);

        // Adds spatial objects to the grid
        if new_object.as_spatial().is_some() {
            if let Some(grid) = &mut self.spatial_grid {
                grid.add_object(&new_object);
            }
        }

        self.global_state.state.changes.extend(new_object.init());

        self.objects.insert(new_id, new_object);

        new_id
    }

    pub fn remove_object(&mut self, id: &Id) {
        if let Some(mut object) = self.objects.remove(id) {
            if let Some(destructable) = object.as_destructible_mut() {
                self.global_state
                    .state
                    .changes
                    .extend(destructable.kill());
            }
            self.remove_indexes(&object);
        }
    }

    pub fn push_event<E: Event>(&mut self, event: E) {
        self.event_bus.push(Box::new(event));
    }

    pub fn sync(&mut self) {
        self.global_state.filtered.clear();

        let stateful_ids = self.indexes.get(&ObjectIndex::Stateful);
        let spatial_ids = self.indexes.get(&ObjectIndex::StatefulSpatial);

        // Spatial states are processed first to protect
        // the grid from non-spatial updates
        if let Some(ids) = spatial_ids {
            for id in ids {
                if let Some(object) = self.objects.get_mut(id) {
                    if let Some(stateful) = object.as_stateful_mut() {
                        self.global_state
                            .state
                            .changes
                            .extend(stateful.state_mut().drain_changes());
                    }
                }
            }

            self.global_state.process();
            
            if let Some(grid) = &mut self.spatial_grid {
                for state in self.global_state.filtered.deletes.iter() {
                    if let StateChange::Delete { occupant, init_pos } = state {
                        grid.remove_cell_occ(*occupant, *init_pos);
                    }
                }
                for state in self.global_state.filtered.updates.iter() {
                    if let StateChange::Update { t_cell, init_pos } = state {
                        if &t_cell.pos != init_pos {
                            grid.remove_cell_occ(t_cell.occ, *init_pos);
                            grid.add_cell_occ(t_cell);
                        }
                    }
                }
                for state in self.global_state.filtered.creates.iter() {
                    if let StateChange::Create { new_t_cell } = state {
                        grid.add_cell_occ(new_t_cell);
                    }
                }
            }
        }


        // Processes the non-spatial states
        match (stateful_ids, spatial_ids) {
            (Some(stateful), Some(spatial)) => {
                for id in stateful.difference(spatial) {
                    if let Some(object) = self.objects.get_mut(id) {
                        if let Some(stateful) = object.as_stateful_mut() {
                            self.global_state
                                .state
                                .changes
                                .extend(stateful.state_mut().drain_changes());
                        }
                    }
                }
            },
            (Some(stateful), None) => {
                for id in stateful {
                    if let Some(object) = self.objects.get_mut(id) {
                        if let Some(stateful) = object.as_stateful_mut() {
                            self.global_state
                                .state
                                .changes
                                .extend(stateful.state_mut().drain_changes());
                        }
                    }
                }
            },
            _ => (),
        }

        self.global_state.process();
    }

    fn add_indexes(&mut self, object: &Box<dyn Object>) {
        let id = object.id();

        if object.as_stateful().is_some() {
            self.indexes
                .entry(ObjectIndex::Stateful)
                .or_default()
                .insert(id);
        }

        if object.as_destructible().is_some() {
            self.indexes
                .entry(ObjectIndex::Destructible)
                .or_default()
                .insert(id);
        }

        if object.as_active().is_some() {
            self.indexes
                .entry(ObjectIndex::Active)
                .or_default()
                .insert(id);
        }

        if object.as_spatial().is_some() {
            self.indexes
                .entry(ObjectIndex::Spatial)
                .or_default()
                .insert(id);
        }

        if object.as_movable().is_some() {
            self.indexes
                .entry(ObjectIndex::Movable)
                .or_default()
                .insert(id);
        }

        if object.as_spatial().is_some() && object.as_spatial().is_some() {
            self.indexes
                .entry(ObjectIndex::StatefulSpatial)
                .or_default()
                .insert(id);
        }
    }

    fn remove_indexes(&mut self, object: &Box<dyn Object>) {
        let id = object.id();

        if object.as_stateful().is_some() {
            if let Some(set) = self.indexes.get_mut(&ObjectIndex::Stateful) {
                set.remove(&id);
            }
        }

        if object.as_destructible().is_some() {
            if let Some(set) = self.indexes.get_mut(&ObjectIndex::Destructible) {
                set.remove(&id);
            }
        }

        if object.as_active().is_some() {
            if let Some(set) = self.indexes.get_mut(&ObjectIndex::Active) {
                set.remove(&id);
            }
        }

        if object.as_spatial().is_some() {
            if let Some(set) = self.indexes.get_mut(&ObjectIndex::Spatial) {
                set.remove(&id);
            }
        }

        if object.as_movable().is_some() {
            if let Some(set) = self.indexes.get_mut(&ObjectIndex::Movable) {
                set.remove(&id);
            }
        }

        if object.as_stateful().is_some() && object.as_spatial().is_some() {
            if let Some(set) = self.indexes.get_mut(&ObjectIndex::StatefulSpatial) {
                set.remove(&id);
            }
        }
    }
}
