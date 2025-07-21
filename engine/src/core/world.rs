use super::global::Id;
use super::object::Object;
use std::collections::{HashMap, HashSet};

pub struct World {
    pub objects: HashMap<Id, Box<dyn Object>>,
    pub consumables_ids: HashSet<Id>,
    pub movable_ids: HashSet<Id>,
}
