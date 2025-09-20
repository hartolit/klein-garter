use crate::prelude::{RuntimeCommand, Scene};
use std::hash::Hash;

pub trait Logic<K: Eq + Hash + Clone> {
    fn dispatch_events(&mut self, scene: &mut Scene);
    fn setup(&mut self, scene: &mut Scene);
    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<K>;
    fn collect_old_stage(
        &mut self,
        _old_scene: Option<Box<Scene>>,
        _old_logic: Option<Box<dyn Logic<K>>>,
    ) {
    }
}
