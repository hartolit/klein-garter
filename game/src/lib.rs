use std::{fmt::{self, Display, Formatter}, time::Duration};

use engine::prelude::{RuntimeManager, Stage};

mod snake_game;

use snake_game::SnakeLogic;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum StageKey {
    Snake,
    Snake1,
}

impl Display for StageKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StageKey::Snake => write!(f, "Snake"),
            StageKey::Snake1 => write!(f, "Snake1"),
        }
    }
}

pub fn init() {
    let mut manager: RuntimeManager<StageKey> = RuntimeManager::new(Duration::from_millis(0));

    let snake_logic = Box::new(SnakeLogic::new(StageKey::Snake));
    let snake_stage: Stage<StageKey> = Stage::new(snake_logic);
    manager.add_stage(StageKey::Snake, snake_stage);

    let snake_logic = Box::new(SnakeLogic::new(StageKey::Snake1));
    let snake_stage: Stage<StageKey> = Stage::new(snake_logic);
    manager.add_stage(StageKey::Snake1, snake_stage);

    manager.set_active_stage(StageKey::Snake);

    manager.run_app();
}
