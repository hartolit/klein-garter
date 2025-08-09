use engine::core::global::{Id, Position};
use engine::core::{Logic, RuntimeCommand, scene::Scene};

pub mod button;

use button::Button;

pub struct UiLogic {
    stashed_scene: Option<Box<Scene>>,
    stashed_logic: Option<Box<dyn Logic>>,
    selected_button_id: usize,
    button_ids: Vec<Id>,
}

impl UiLogic {
    pub fn new() -> Self {
        Self {
            stashed_scene: None,
            stashed_logic: None,
            selected_button_id: 0,
            button_ids: Vec::new(),
        }
    }
}

impl Logic for UiLogic {
    fn setup(&self, scene: &mut Scene) {
        let start_button_id = scene.attach_object(|id| {
            Box::new(Button::new(id, Position::new(0, 0), String::from("Test")))
        });
    }

    fn update(&self, scene: &mut Scene) -> RuntimeCommand {
        RuntimeCommand::None
    }

    fn process_actions(&self, scene: &mut Scene, actions: Vec<engine::core::object::Action>) {}

    fn process_input(&self) {}

    fn collect_old_stage(
        &mut self,
        _old_scene: Option<Box<Scene>>,
        _old_logic: Option<Box<dyn Logic>>,
    ) {
    }
}
