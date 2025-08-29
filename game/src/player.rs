use std::collections::HashMap;

use ::engine::core::global::Id;

use super::snake::Direction;
use uuid::Uuid;

#[derive(PartialEq, Eq)]
pub enum PlayerKind {
    Local,
    Online,
}

#[derive(PartialEq, Eq)]
pub struct Player {
    pub id: Uuid,
    pub score: u16,
    pub snake: Option<Id>,
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

    pub fn set_snake(&mut self, snake_id: Id) {
        self.snake = Some(snake_id)
    }
}
