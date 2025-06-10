mod snake;

pub use snake::{Direction, Snake};

use crate::game::global::Position;

pub struct Player {
    pub score: u16,
    pub snake: Snake,
    pub input: Option<[u8; 1]>,
}

impl Player {
    pub fn new(pos: Position) -> Self {
        Player { 
            score: 0, 
            snake: Snake::new(pos), 
            input: None 
        }
    }
}