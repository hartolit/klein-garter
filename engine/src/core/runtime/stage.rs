use std::hash::Hash;

pub mod logic;
pub mod scene;

pub use logic::Logic;
pub use scene::Scene;

pub struct Stage<K: Eq + Hash + Clone> {
    pub logic: Box<dyn Logic<K>>,
    pub scene: Box<Scene>,
    pub is_init: bool,
}

impl<K: Eq + Hash + Clone> Stage<K> {
    pub fn new(logic: Box<dyn Logic<K>>) -> Self {
        Self {
            logic,
            scene: Box::new(Scene::new()),
            is_init: false,
        }
    }

    pub fn replace_scene(&mut self, scene: Box<Scene>) -> Box<Scene> {
        let old_scene = std::mem::replace(&mut self.scene, scene);
        old_scene
    }

    pub fn replace_logic(&mut self, logic: Box<dyn Logic<K>>) -> Box<dyn Logic<K>> {
        let old_logic = std::mem::replace(&mut self.logic, logic);
        old_logic
    }
}
