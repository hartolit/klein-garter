use std::collections::{HashMap, HashSet};

use crate::core::global::IdCounter;

use super::global::Id;
use super::object::Object;
use super::grid::SpatialGrid;

pub struct World {
    pub id_counter: IdCounter,
    pub objects: HashMap<Id, Box<dyn Object>>,
    pub consumables_ids: HashSet<Id>,
    pub damaging_ids: HashSet<Id>,
    pub movable_ids: HashSet<Id>,
    pub grid: SpatialGrid,
}

