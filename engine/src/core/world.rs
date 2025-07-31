use std::collections::{HashMap, HashSet};

use super::global::{Id, IdCounter, Position};
use super::grid::SpatialGrid;
use super::object::{Action, Object};

// TODO - Move to a proper ECS architecture? (future improvements)
pub struct World {
    pub id_counter: IdCounter,
    pub objects: HashMap<Id, Box<dyn Object>>,
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

    pub fn tick(objects: &mut HashMap<Id, Box<dyn Object>>, spatial_grid: &SpatialGrid) {
        let future_moves = objects
            .iter()
            .filter_map(|(id, object)| object.as_movable().map(|m| (id, m)))
            .flat_map(|(id, movable)| movable.predict_pos().map(move |pos| (*id, pos)));

        let mut collision_map = spatial_grid.probe_moves(future_moves);

        let mut actions: Vec<Action> = Vec::new();

        for (id, collisions) in collision_map.drain() {
            if let Some(object) = objects.get_mut(&id) {
                if let Some(movable) = object.as_movable_mut() {
                    actions.extend(movable.make_move(collisions));
                }
            }
        }

        // Collect state changes

        for action in actions {}
    }
}
