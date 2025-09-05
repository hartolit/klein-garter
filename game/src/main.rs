use std::time::Duration;

use engine::core::{RuntimeManager, Stage};

mod snake_game;

use snake_game::GameLogic;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum StageKey {
    SnakeGame,
}


fn main() {
    let mut manager: RuntimeManager<StageKey> = RuntimeManager::new(Duration::from_millis(0));

    let game_logic = Box::new(GameLogic::new());
    let game_stage:Stage<StageKey> = Stage::new(game_logic);

    manager.add_stage(StageKey::SnakeGame, game_stage);
    manager.set_active_stage(StageKey::SnakeGame);

    manager.run_app();
}