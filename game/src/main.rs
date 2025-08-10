use std::time::Duration;

use engine::core::RuntimeManager;

mod game;
mod ui;

fn main() {
    let mut manager: RuntimeManager<String> = RuntimeManager::new(Duration::new(0, 60));
    manager.run_app();
}
