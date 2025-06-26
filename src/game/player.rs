mod snake;

use std::{collections::HashMap};

pub use snake::{Direction, Snake};

use crate::game::global::Position;
use uuid::Uuid;

pub enum PlayerKind {
    Local,
    Online,
}

pub struct Player {
    pub id: Uuid,
    pub score: u16,
    pub snake: Snake,
    pub kind: PlayerKind,
    pub keys: HashMap<Direction, char>,
    pub input: Option<[u8; 1]>,
}

impl Player {
    pub fn new(kind: PlayerKind, keys: HashMap<Direction, char>) -> Self {
        Player { 
            id: Uuid::new_v4(),
            score: 0, 
            snake: Snake::new(Position { x: 0, y: 0 }), // Is calculated in game.init()
            kind,
            keys,
            input: None 
        }
    }
}