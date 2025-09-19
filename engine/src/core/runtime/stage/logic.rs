use std::hash::Hash;
use crate::prelude::{Scene, RuntimeCommand};

pub trait Logic<K: Eq + Hash + Clone> {
    fn process_events(&mut self, scene: &mut Scene);
    fn process_input(&mut self, scene: &mut Scene);
    fn setup(&mut self, scene: &mut Scene);
    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<K>;
    fn collect_old_stage(
        &mut self,
        _old_scene: Option<Box<Scene>>,
        _old_logic: Option<Box<dyn Logic<K>>>,
    ) {
    }
}