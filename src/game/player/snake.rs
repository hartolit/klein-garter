use std::{collections::VecDeque};

use crate::game::{object::{Collision, DynamicObject, Element, Glyph, Object, ObjectId, Position, StateChange}};
use crossterm::style::Color;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Snake {
    id: ObjectId,
    size: usize,
    pub is_alive: bool,
    pub meals: i16,
    pub head: Vec<Element>,
    pub body: VecDeque<Vec<Element>>,
    pub head_style: Glyph,
    pub body_style: Glyph,
    pub direction: Direction,
}

impl Snake {
    pub fn new(pos: Position, obj_id: ObjectId, size: usize) -> Self {
        let head_style = Glyph { fg_clr: Some(Color::DarkMagenta), bg_clr: None, symbol: '█' };
        let body_style = Glyph { fg_clr: Some(Color::DarkYellow), bg_clr: None, symbol: 'S' };

        let mut snake = Snake {
            id: obj_id,
            size: 1, // Start as 1x1
            is_alive: true,
            meals: 1,
            head: vec![Element::new(head_style, Some(pos))],
            body: VecDeque::new(),
            head_style,
            body_style,
            direction: Direction::Down,
        };

        snake.set_size(size);
        snake
    }

    pub fn set_size (&mut self, new_size: usize) {
        let odd_size = if new_size % 2 == 0 { new_size.saturating_sub(1).max(1) } else { new_size };
        let top_left = {
            let tmp_pos = self.head.first().expect("Head cannot be empty.").pos;

            let center_pos = Position {
                x: tmp_pos.x + self.size as u16 / 2,
                y: tmp_pos.y + self.size as u16 / 2,
            };

            let top_left = Position {
                x: center_pos.x.saturating_sub(odd_size as u16 / 2),
                y: center_pos.y.saturating_sub(odd_size as u16 / 2)
            };

            top_left
        };

        self.size = odd_size;
        self.head.clear();

        for row in 0..odd_size {
            for col in 0..odd_size {
                let curr_pos = Position {
                    x: top_left.x + col as u16,
                    y: top_left.y + row as u16,
                };
                self.head.push(Element::new(self.head_style, Some(curr_pos)));
            }
        }
    }
}

impl Object for Snake {
    fn id(&self) -> ObjectId {
        self.id
    }

    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_> {
        Box::new(self.head.iter().chain(self.body.iter().flatten()))
    }

    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_> {
        Box::new(
            Box::new(
                self.head.iter().map(|e| e.pos)
                .chain(self.body.iter().flatten().map(|e| e.pos)))
        )
    }
}

impl DynamicObject for Snake {
    fn next_pos(&self) -> Box<dyn Iterator<Item = Position> + '_> {
        let (dx, dy) = match self.direction {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };

        Box::new(self.head.iter().map(move |elem| Position {
            x: (elem.pos.x as i16 + dx) as u16,
            y: (elem.pos.y as i16 + dy) as u16,
        }))
    }

    fn update(&mut self, collisions: Option<Vec<Collision>>) -> Option<Vec<StateChange>> {
        ()
    }
}