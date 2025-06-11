use std::collections::{VecDeque};

use crate::game::global::{Position};
use crossterm::style::Color;

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Snake {
    pub is_alive: bool,
    pub meals: i16,
    pub head: char,
    pub body: char,
    pub head_pos: Position,
    pub body_vec: VecDeque<Position>,
    pub head_color: Color,
    pub body_color: Color,
    pub direction: Direction,
}

impl Snake {
    pub fn new(pos: Position) -> Self {        
        return Self {
            is_alive: true,
            meals: 50,
            head: '\u{25FC}',
            body: '\u{2588}',
            head_pos: pos,
            body_vec: VecDeque::new(),
            head_color: Color::Rgb { r: 50, g: 80, b: 120 },
            body_color: Color::Rgb { r: 200, g: 200, b: 200 },
            direction: Direction::Down,
        };
    }

    pub fn slither(&mut self) {
        match self.direction {
            Direction::Up => self.head_pos.y -= 1,
            Direction::Down => self.head_pos.y += 1,
            Direction::Left => self.head_pos.x -= 1,
            Direction::Right => self.head_pos.x += 1,
        };

        // Collision check of body
        for part in &self.body_vec {
            if self.head_pos == *part {
                self.is_alive = false;
            }
        }
    }
}