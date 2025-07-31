use std::collections::{HashMap, HashSet};

use crate::core::object::state::StateChange;
use crate::core::GameLogic;

use super::global::{Id, IdCounter, Position};
use super::grid::SpatialGrid;
use super::object::{Action, Object};

// TODO - Move to a proper ECS architecture? (future improvements)
pub struct World {
    pub id_counter: IdCounter,
    pub objects: HashMap<Id, Box<dyn Object>>,
    pub movable_ids: HashSet<Id>,
    pub killed_objects: HashSet<Id>,
    pub spatial_grid: SpatialGrid,
}

impl World {
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
        if self.objects.remove(id).is_some() {
            self.movable_ids.remove(id);
        }
    }

    fn draw(changes: Vec<StateChange>) {

    }
}
