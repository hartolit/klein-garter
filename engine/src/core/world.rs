use std::collections::{HashMap, HashSet};

use super::global::{Id, IdCounter};
use super::grid::SpatialGrid;
use super::object::{Object, state::StateManager};

pub struct World {
    pub id_counter: IdCounter,
    pub objects: HashMap<Id, Box<dyn Object>>,
    pub movable_ids: HashSet<Id>,
    pub spatial_grid: SpatialGrid,
    pub global_state: StateManager,
}

impl World {
    pub fn new(spatial_grid: SpatialGrid) -> Self {
        Self {
            id_counter: IdCounter::new(),
            objects: HashMap::new(),
            movable_ids: HashSet::new(),
            spatial_grid,
            global_state: StateManager::new(),
        }
    }

    pub fn attach_object<F>(&mut self, create_fn: F) -> Id
    where
        F: FnOnce(Id) -> Box<dyn Object>,
    {
        let new_id = self.id_counter.next();
        let new_object = create_fn(new_id);

        if new_object.as_movable().is_some() {
            self.movable_ids.insert(new_id);
        }

        self.objects.insert(new_id, new_object);
        new_id
    }

    pub fn remove_object(&mut self, id: &Id) {
        if let Some(mut object) = self.objects.remove(id) {
            if let Some(destructable) = object.as_destructible_mut() {
                destructable.kill();
                self.global_state.changes.extend(destructable.state_manager_mut().drain_changes());
                if let Some(_) = object.as_movable() {
                    self.movable_ids.remove(id);
                }
            }
        }
    }
}
