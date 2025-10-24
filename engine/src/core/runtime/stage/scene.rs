use std::collections::{HashMap, HashSet};

pub mod global_state;
pub mod grid;
pub mod object;

use global_state::GlobalState;

use crate::prelude::{Event, Id, IdCounter, Object, SpatialGrid, StateChange};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum ObjectIndex {
    Stateful,
    Destructible,
    Active,
    Spatial,
    Movable,
    StatefulSpatial,
}

pub enum Conflict {
    Overwrite,
    Ignore,
    Cancel,
}

pub struct Scene {
    pub id_counter: IdCounter,
    pub objects: HashMap<Id, Box<dyn Object>>,
    pub indexes: HashMap<ObjectIndex, HashSet<Id>>,
    pub protected_ids: HashSet<Id>,
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
            protected_ids: HashSet::new(),
            spatial_grid: None,
            global_state: GlobalState::new(),
            event_bus: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.id_counter.reset();
        self.objects.clear();
        self.indexes.clear();
        self.protected_ids.clear();
        self.spatial_grid = None;
        self.global_state.clear();
        self.event_bus.clear();
    }

    pub fn attach_grid(&mut self, grid: SpatialGrid) {
        self.spatial_grid = Some(grid);
        self.resync_grid();
    }

    pub fn attach_object<F>(&mut self, create_fn: F, on_conflict: Conflict) -> Option<Id>
    where
        F: FnOnce(Id) -> Box<dyn Object>,
    {
        let new_id = self.id_counter.next();
        let new_object = create_fn(new_id);

        // Special logic for grid objects <3
        if new_object.as_spatial().is_some() {
            // Probes grid for conflicts
            let mut collisions: HashSet<Id> = HashSet::new();
            if let Some(grid) = &self.spatial_grid {
                if !grid.check_bounds(&new_object) {
                    return None;
                }
                collisions = grid.probe_object(&new_object);
            }

            if !collisions.is_empty() {
                // Checks if collisions are protected
                let is_protected_collision =
                    collisions.iter().any(|id| self.protected_ids.contains(id));

                // Cancels Object creation
                if is_protected_collision {
                    return None;
                }

                // Handles different conflict solutions
                match on_conflict {
                    Conflict::Cancel => return None,
                    Conflict::Overwrite => {
                        for id in collisions {
                            self.remove_object(&id);
                        }
                    }
                    Conflict::Ignore => {}
                }
            }

            if let Some(grid) = &mut self.spatial_grid {
                grid.add_object(&new_object);
            }
        }

        self.index_object(&new_object, true);
        self.global_state.state.changes.extend(new_object.init());
        self.objects.insert(new_id, new_object);
        Some(new_id)
    }

    pub fn remove_object(&mut self, id: &Id) {
        if let Some(mut object) = self.objects.remove(id) {
            self.protected_ids.remove(&id);
            if let Some(destructable) = object.as_destructible_mut() {
                self.global_state.state.changes.extend(destructable.kill());
            }
            self.index_object(&object, false);
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
        // and filter the grid from non-spatial updates
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

            // Process spatial states
            self.global_state.process(true);

            if let Some(grid) = &mut self.spatial_grid {
                // Keeps only valid grid changes
                self.global_state
                    .filtered
                    .spatial
                    .retain(|state| match state {
                        StateChange::Delete { occupant, init_pos } => {
                            grid.remove_cell_occ(*occupant, *init_pos)
                        }
                        StateChange::Create { new_t_cell } => grid.add_cell_occ(new_t_cell),
                        StateChange::Update { t_cell, init_pos } => {
                            if &t_cell.pos != init_pos {
                                // Ignores false removal as it would have no impact.
                                grid.remove_cell_occ(t_cell.occ, *init_pos);
                                grid.add_cell_occ(t_cell)
                            } else {
                                // In-place update.
                                grid.add_cell_occ(t_cell)
                            }
                        }
                    });
            }
        }

        // Non-spatial states
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
            }
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
            }
            _ => (),
        }

        self.global_state.process(false);
    }

    pub fn resync_grid(&mut self) {
        if let Some(grid) = &mut self.spatial_grid {
            grid.clear();

            let spatial_ids = self.indexes.get(&ObjectIndex::Spatial);
            if let Some(ids) = spatial_ids {
                for id in ids {
                    if let Some(object) = self.objects.get(id) {
                        grid.add_object(object);
                    }
                }
            }
        }
    }
    
    pub fn set_overwrite_exemption(&mut self, id: Id, is_exempt: bool) {
        if is_exempt {
            self.protected_ids.insert(id);
        } else {
            self.protected_ids.remove(&id);
        }
    }

    fn index_object(&mut self, object: &Box<dyn Object>, is_insert: bool) {
        let id = object.id();

        let checks = [
            (object.as_stateful().is_some(), ObjectIndex::Stateful),
            (
                object.as_destructible().is_some(),
                ObjectIndex::Destructible,
            ),
            (object.as_active().is_some(), ObjectIndex::Active),
            (object.as_spatial().is_some(), ObjectIndex::Spatial),
            (object.as_movable().is_some(), ObjectIndex::Movable),
            (
                object.as_stateful().is_some() && object.as_spatial().is_some(),
                ObjectIndex::StatefulSpatial,
            ),
        ];

        for (has_trait, index) in checks {
            if has_trait {
                if is_insert {
                    self.indexes.entry(index).or_default().insert(id);
                } else {
                    if let Some(hash_set) = self.indexes.get_mut(&index) {
                        hash_set.remove(&id);
                    }
                }
            }
        }
    }
}
