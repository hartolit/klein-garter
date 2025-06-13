mod game;

use game::{Game, GameMode};

fn main() {
    let mut game = Game::new(GameMode::Singleplayer);

    game.start();
}

