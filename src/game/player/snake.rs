mod animation;

use std::{collections::VecDeque};
use crossterm::style::Color;

use crate::game::food;
use crate::game::grid::{CellKind, ObjectRef};
use crate::game::object::{Collision, DynamicObject, Element, Glyph, Object, ObjectId, Position, StateChange};
use animation::{Effect};

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
    tmp_size: Option<usize>,
    effect: Option<Effect>,
    pub is_alive: bool,
    pub meals: i16,
    pub head: VecDeque<Vec<Element>>,
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
            tmp_size: None,
            effect: None,
            is_alive: true,
            meals: 1,
            head: VecDeque::from([vec![Element::new(head_style, Some(pos))]]),
            body: VecDeque::new(),
            head_style,
            body_style,
            direction: Direction::Down,
        };

        snake.resize_head(size);
        snake
    }

    pub fn resize_head(&mut self, new_size: usize) {
        let resize = self.set_size(new_size, true);
        self.head.clear();
        self.head = resize;
    }

    pub fn resize_head_tmp(&mut self, new_size: usize) {
        let resize = self.set_size(new_size, true);
        self.tmp_size = Some(new_size);
        self.head.clear();
        self.head = resize;
    }

    pub fn resize_head_restore(&mut self) {
        let resize = self.set_size(self.size, true);
        if let Some(_) = self.tmp_size { self.tmp_size = None; }
        self.head.clear();
        self.head = resize;
    }

    pub fn get_resized_body_part(&mut self, new_size: usize) -> Option<Vec<Element>> {
        let mut resize = self.set_size(new_size, false);
        return resize.pop_back();
    }

    fn set_size(&mut self, new_size: usize, is_head: bool) -> VecDeque<Vec<Element>> {
        let odd_size = if new_size % 2 == 0 { new_size.saturating_sub(1).max(1) } else { new_size };
        let new_buttom_left = {
            let tmp_pos = self.head.back().expect("Missing head vector").first().expect("Missing head element").pos;

            let center_pos = Position {
                x: tmp_pos.x + self.size as u16 / 2,
                y: tmp_pos.y + self.size as u16 / 2,
            };

            let buttom_left = Position {
                x: center_pos.x.saturating_sub(odd_size as u16 / 2),
                y: center_pos.y.saturating_sub(odd_size as u16 / 2)
            };

            buttom_left
        };

        let mut new_elements: VecDeque<Vec<Element>> = VecDeque::new();
        for row in 0..odd_size {
            let mut elements: Vec<Element> = Vec::new();
            for col in 0..odd_size {
                let curr_pos = Position {
                    x: new_buttom_left.x + col as u16,
                    y: new_buttom_left.y - row as u16, // Drawing from buttom and up
                };
                elements.push(Element::new(if is_head { self.head_style } else { self.body_style }, Some(curr_pos)));
            }
            new_elements.push_back(elements);
        }

        new_elements
    }
}

impl Object for Snake {
    fn id(&self) -> ObjectId {
        self.id
    }

    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_> {
        Box::new(self.head.iter().flatten().chain(self.body.iter().flatten()))
    }

    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_> {
        Box::new(
            Box::new(
                self.head.iter().flatten().map(|e| e.pos)
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

        Box::new(self.head.iter().flatten().map(move |elem| Position {
            x: (elem.pos.x as i16 + dx) as u16,
            y: (elem.pos.y as i16 + dy) as u16,
        }))
    }

    fn update(&mut self, collisions: Option<Vec<Collision>>) -> Option<Vec<StateChange>> {
        if !self.is_alive {
            return None;
        }

        let mut state_changes: Vec<StateChange> = Vec::new();
        let mut new_effect: Option<Effect> = None;

        if let Some(cols) = collisions {
            for col in cols {
                if let CellKind::Border | CellKind::Lava = col.kind {
                    self.is_alive = false;
                    new_effect = Some(Effect::new(1, animation::Kind::Damage, None))
                }

                for obj_ref in col.colliders {
                    match obj_ref {
                        ObjectRef::Food(food_id, food_kind, meals) => {
                            match &food_kind {
                                food::Kind::Bomb => new_effect = Some(Effect::new(3, animation::Kind::Damage, Some(self.size + 2))),
                                food::Kind::Cherry => new_effect = Some(Effect::new(3, animation::Kind::Damage, Some(self.size + 2))),
                                food::Kind::Mouse => new_effect = Some(Effect::new(3, animation::Kind::Damage, Some(self.size + 2))),
                            }
                            self.meals += meals;
                            state_changes.push(StateChange::new(*food_id, col.pos, None));
                        },
                        ObjectRef::Player(_) => {
                            self.is_alive = false;
                            new_effect = Some(Effect::new(1, animation::Kind::Damage, None))
                        }
                    }
                }
            }
        }

        // Move head

        // Add effects
        if let Some(effect) = new_effect {
            self.effect = Some(effect);
            if let Some(size) = effect.action_size {
                self.resize_head_tmp(size);
            }
        }

        if let Some(effect) = self.effect {
            if let Some(size) = effect.action_size {
                self.resize_head_tmp(size);
            }
        }

        // Draw body

        return Some(state_changes);
    }
}