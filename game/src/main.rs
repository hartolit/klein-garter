use crossterm::{QueueableCommand, terminal};
use engine::core::GameLogic;
use std::{collections::HashMap, io::stdout};

mod bomb;
mod food;
mod game;
mod player;
mod snake;

use player::{Player, PlayerKind};
use snake::Direction;

fn main() {
    let mut out = stdout();

    out.queue(terminal::SetTitle(format!("Klein Garter")));
    out.queue(terminal::SetSize(100, 100));

    // let mut game = Game::new(GameKind::Local, &mut out);

    // game.players.push(Player::new(
    //     PlayerKind::Local,
    //     HashMap::from([
    //         (Direction::Up, 'w'),
    //         (Direction::Down, 's'),
    //         (Direction::Left, 'a'),
    //         (Direction::Right, 'd'),
    //     ]),
    // ));

    // game.players.push(Player::new(
    //     PlayerKind::Local,
    //     HashMap::from([
    //         (Direction::Up, 'w'),
    //         (Direction::Down, 's'),
    //         (Direction::Left, 'a'),
    //         (Direction::Right, 'd'),
    //     ]),
    // ));

    // game.start();
}
