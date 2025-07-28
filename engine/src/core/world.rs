use std::collections::{HashMap, HashSet};

use super::global::{Id, IdCounter};
use super::grid::{SpatialGrid, Collision};
use super::object::Object;

// TODO - Move to a proper ECS architecture? (future improvements)
pub struct World {
    pub id_counter: IdCounter,
    pub objects: HashMap<Id, Box<dyn Object>>,
    pub consumables_ids: HashSet<Id>,
    pub damaging_ids: HashSet<Id>,
    pub movable_ids: HashSet<Id>,
    pub spatial_grid: SpatialGrid,
}

impl World {
    pub fn attach_object<F>(&mut self, create_fn: F) -> Id
    where
        F: FnOnce(Id) -> Box<dyn Object>,
    {
        let new_id = self.id_counter.next();
        let new_object = create_fn(new_id);

        if new_object.as_consumable().is_some() {
            self.consumables_ids.insert(new_id);
        }
        if new_object.as_movable().is_some() {
            self.movable_ids.insert(new_id);
        }
        if new_object.as_damaging().is_some() {
            self.damaging_ids.insert(new_id);
        }

        self.objects.insert(new_id, new_object);

        new_id
    }

    pub fn remove_object(&mut self, id: &Id) {
        if self.objects.remove(id).is_some() {
            self.consumables_ids.remove(id);
            self.movable_ids.remove(id);
            self.damaging_ids.remove(id);
        }
    }

    pub fn tick(&mut self) {
        for object in self.objects.values() {
            if let Some(movable) = object.as_movable() {
                let collisions: Box<dyn Iterator<Item = Collision<'_>>> = self.spatial_grid.get_collisions(movable.next_pos());
                let changes = movable.update(collisions, &mut self.objects);
            }
        }
    }
}
