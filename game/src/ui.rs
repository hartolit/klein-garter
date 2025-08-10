use std::hash::Hash;
use engine::core::global::{Id, Position};
use engine::core::{Logic, RuntimeCommand, scene::Scene};

pub mod button;

use button::Button;

pub struct MainMenuLogic<K: Eq + Hash + Clone> {
    selected_button_id: usize,
    button_ids: Vec<Id>,
}

impl<K: Eq + Hash + Clone> MainMenuLogic<K> {
    pub fn new() -> Self {
        Self {
            selected_button_id: 0,
            button_ids: Vec::new(),
        }
    }
}

impl<K: Eq + Hash + Clone> Logic<K> for MainMenuLogic<K> {
    fn setup(&self, scene: &mut Scene) {
        let start_button_id = scene.attach_object(|id| {
            Box::new(Button::new(id, Position::new(0, 0), String::from("Test")))
        });
    }

    fn update(&self, scene: &mut Scene) -> RuntimeCommand<K> {
        RuntimeCommand::None
    }

    fn process_actions(&self, scene: &mut Scene, actions: Vec<engine::core::object::Action>) {}

    fn process_input(&self) {}
}
