pub struct World {
    pub objects: HashMap<Id, Box<dyn Object>>,
    pub consumables_ids: HashSet<Id>,
    pub movable_ids: HashSet<Id>,
}