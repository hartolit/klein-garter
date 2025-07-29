use std::collections::{HashMap, HashSet};

use super::global::{Id, IdCounter, Position};
use super::grid::{SpatialGrid};
use super::object::Object;

// TODO - Move to a proper ECS architecture? (future improvements)
pub struct World<'a> {
    pub id_counter: IdCounter,
    pub objects: HashMap<Id, Box<dyn Object<'a>>>,
    pub consumables_ids: HashSet<Id>,
    pub damaging_ids: HashSet<Id>,
    pub movable_ids: HashSet<Id>,
    pub spatial_grid: SpatialGrid,
}

impl<'a> World<'a> {
    pub fn attach_object<F>(&mut self, create_fn: F) -> Id
    where
        F: FnOnce(Id) -> Box<dyn Object<'a>>,
    {
        let new_id = self.id_counter.next();
        let mut new_object = create_fn(new_id);

        if new_object.as_consumable().is_some() {
            self.consumables_ids.insert(new_id);
        }
        if new_object.as_movable_mut().is_some() {
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

    pub fn tick(objects: &mut HashMap<Id, Box<dyn Object<'a>>>, spatial_grid: &'a SpatialGrid) {
        for object in objects.values_mut() {
            if let Some(movable) = object.as_movable_mut() {
                let next_move: Vec<Position> = movable.next_pos().collect(); // Collects to prevent inner iterator borrow issues
                let collisions = spatial_grid.get_collisions(Box::new(next_move.into_iter()));
                movable.update(collisions);
            }
        }
    }
}