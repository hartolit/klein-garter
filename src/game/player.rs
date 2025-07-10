mod snake;

use std::collections::HashMap;

pub use snake::{Direction, Snake};

use crate::game::object::{Id, Position};
use uuid::Uuid;

pub enum PlayerKind {
    Local,
    Online,
}

pub struct Player {
    pub id: Uuid,
    pub score: u16,
    pub snake: Option<Snake>,
    pub kind: PlayerKind,
    pub keys: HashMap<Direction, char>,
}

impl Player {
    pub fn new(kind: PlayerKind, keys: HashMap<Direction, char>) -> Self {
        Player {
            id: Uuid::new_v4(),
            score: 0,
            snake: None,
            kind,
            keys,
        }
    }

    pub fn add_snake(&mut self, pos: Position, obj_id: Id, size: usize) {
        self.snake = Some(Snake::new(pos, obj_id, size));
    }
}
