mod game;

use crossterm::{QueueableCommand, terminal};
use std::{collections::HashMap, io::stdout};

use game::{
    Game, GameKind,
    player::{Player, PlayerKind},
};

use crate::game::player::Direction;

fn main() {
    let mut out = stdout();

    out.queue(terminal::SetTitle(format!("Klein Garter")));
    out.queue(terminal::SetSize(100, 100));

    let mut game = Game::new(GameKind::Local, &mut out);

    game.players.push(Player::new(
        PlayerKind::Local,
        HashMap::from([
            (Direction::Up, 'w'),
            (Direction::Down, 's'),
            (Direction::Left, 'a'),
            (Direction::Right, 'd'),
        ]),
    ));

    game.players.push(Player::new(
        PlayerKind::Local,
        HashMap::from([
            (Direction::Up, 'w'),
            (Direction::Down, 's'),
            (Direction::Left, 'a'),
            (Direction::Right, 'd'),
        ]),
    ));

    game.start();
}
