use std::time::Duration;

use engine::prelude::{RuntimeManager, Stage};

mod snake_game;

use snake_game::SnakeLogic;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum StageKey {
    Snake,
    Snake1,
}

fn main() {
    let mut manager: RuntimeManager<StageKey> = RuntimeManager::new(Duration::from_millis(0));

    let snake_logic = Box::new(SnakeLogic::new());
    let snake_stage: Stage<StageKey> = Stage::new(snake_logic);
    manager.add_stage(StageKey::Snake, snake_stage);
    
    let snake_logic = Box::new(SnakeLogic::new());
    let snake_stage: Stage<StageKey> = Stage::new(snake_logic);
    manager.add_stage(StageKey::Snake1, snake_stage);

    manager.set_active_stage(StageKey::Snake);

    manager.run_app();
}
