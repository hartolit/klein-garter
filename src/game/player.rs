mod snake;

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
    pub input: Option<[u8; 1]>,
}

impl Player {
    pub fn new(input_type: PlayerKind, pos: Position) -> Self {
        Player { 
            id: Uuid::new_v4(),
            score: 0, 
            snake: Snake::new(pos), 
            kind: input_type,
            input: None 
        }
    }
}