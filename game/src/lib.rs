use std::{fmt::{self, Display, Formatter}, time::Duration};

use engine::prelude::{RuntimeManager, Stage};

mod snake_game;

use snake_game::SnakeLogic;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum StageKey {
    Level0,
    Level1,
}

impl Display for StageKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StageKey::Level0 => write!(f, "Level 0"),
            StageKey::Level1 => write!(f, "Level 1"),
        }
    }
}

pub fn init() {
    let mut manager: RuntimeManager<StageKey> = RuntimeManager::new(Duration::from_millis(0));

    let snake_logic = Box::new(SnakeLogic::new(StageKey::Level0));
    let snake_stage: Stage<StageKey> = Stage::new(snake_logic);
    manager.add_stage(StageKey::Level0, snake_stage);

    let snake_logic = Box::new(SnakeLogic::new(StageKey::Level1));
    let snake_stage: Stage<StageKey> = Stage::new(snake_logic);
    manager.add_stage(StageKey::Level1, snake_stage);

    manager.set_active_stage(StageKey::Level0);

    manager.run_app();
}
