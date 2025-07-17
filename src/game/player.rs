use std::collections::HashMap;

use super::object::Id;
use super::snake::Direction;
use uuid::Uuid;

pub enum PlayerKind {
    Local,
    Online,
}

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
}
