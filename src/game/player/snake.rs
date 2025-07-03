mod animation;

use std::{collections::VecDeque};
use crossterm::style::Color;

use crate::game::player::snake::animation::EffectZone;
use crate::game::{food};
use crate::game::grid::{CellKind, ObjectRef};
use crate::game::object::{Collision, DynamicObject, Element, Glyph, Object, ObjectId, Position, StateChange};
use animation::{Effect, EffectStyle};

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
    size_tmp: Option<usize>,
    effect: Option<Effect>,
    is_alive: bool,
    meals: i16,
    head: VecDeque<Vec<Element>>,
    body: VecDeque<Vec<Element>>,
    head_style: Glyph,
    body_style: Glyph,
    direction: Direction,
}

impl Snake {
    pub fn new(pos: Position, obj_id: ObjectId, size: usize) -> Self {
        let head_style = Glyph { fg_clr: Some(Color::DarkMagenta), bg_clr: None, symbol: '█' };
        let body_style = Glyph { fg_clr: Some(Color::DarkYellow), bg_clr: None, symbol: 'S' };

        let mut snake = Snake {
            id: obj_id,
            size: 1, // Start as 1x1
            size_tmp: None,
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

    fn resize_head(&mut self, new_size: usize) {
        // Skip unnecessary resize call
        if None == self.size_tmp && self.size == new_size {
            return;
        } else if let Some(tmp_size) = self.size_tmp && tmp_size == new_size {
            self.size = tmp_size;
            self.size_tmp = None;
            return;
        }

        let resize = self.set_size(new_size, true);

        self.head.clear();
        self.head = resize;

        // Reset temporary size (no longer valid)
        if let Some(_) = self.size_tmp {
            self.size_tmp = None; 
        }
    }

    fn resize_head_tmp(&mut self, new_size: usize) {
        // Skip unnecessary resize call
        if None == self.size_tmp && self.size == new_size || matches!(&self.size_tmp, Some(tmp_size) if *tmp_size == new_size){
            return;
        }

        let resize = self.set_size(new_size, true);
    
        self.head.clear();
        self.head = resize;
        
        self.size_tmp = Some(new_size);
    }

    fn resize_head_restore(&mut self) {
        // Skip unnecessary resize call
        if None == self.size_tmp {
            return;
        }

        let resize = self.set_size(self.size, true);

        self.head.clear();
        self.head = resize;
        
        // Reset temporary size (no longer valid)
        self.size_tmp = None; 
    }

    fn get_resized_body_part(&mut self, new_size: usize) -> Vec<Element> {
        let mut resize = self.set_size(new_size, false);
        return if let Some(back_row) = resize.pop_back() {
            back_row } else { return Vec::new(); };
    }

    fn set_size(&mut self, new_size: usize, is_head: bool) -> VecDeque<Vec<Element>> {
        let odd_size = if new_size % 2 == 0 { new_size.saturating_sub(1).max(1) } else { new_size };
        let new_buttom_left = {
            let tmp_pos = self.head.back().expect("Missing head vector!").first().expect("Missing head element!").pos;

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

        let rows_iter = if is_head { odd_size } else { 1 };

        let mut new_elements: VecDeque<Vec<Element>> = VecDeque::new();
        for row in 0..rows_iter {
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

    fn slither_head (&mut self) -> Vec<StateChange> {
        let mut changes: Vec<StateChange> = Vec::new();

        let (dx, dy) = match self.direction {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };

        let mut new_row = if let Some(front_row) = self.head.front() {
            front_row.clone() } else { return changes; };

        for e in new_row.iter_mut() {
            e.pos = Position::new((e.pos.x as i16 + dx) as u16, (e.pos.y as i16 + dy) as u16)
        }

        if let Some(back_row) = self.head.pop_back() {
            for e in back_row.iter().enumerate() {
                changes.push(StateChange::new(self.id, e.1.pos, new_row.get(e.0).copied()));
            }
        }

        self.head.push_front(new_row);

        changes
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
        let mut is_resized = false; // Used to redraw head

        if let Some(cols) = collisions {
            for col in cols {
                if let CellKind::Border | CellKind::Lava = col.kind {
                    self.is_alive = false;
                    new_effect = Some(Effect::new(1, EffectStyle::Damage, None, EffectZone::All))
                }

                for obj_ref in col.colliders {
                    match obj_ref {
                        ObjectRef::Food { obj_id, kind, meals } => {
                            match &*kind {
                                food::Kind::Bomb => new_effect = Some(Effect::new(2, EffectStyle::Damage, Some(self.size + 2), EffectZone::All)),
                                food::Kind::Cherry => new_effect = Some(Effect::new(2, EffectStyle::Grow, Some(self.size + 2), EffectZone::Body)),
                                food::Kind::Mouse => new_effect = Some(Effect::new(2, EffectStyle::Grow, Some(self.size + 2), EffectZone::Body)),
                                food::Kind::Grower => { self.resize_head(self.size + 2); is_resized = true },
                            }
                            self.meals += meals;

                            if self.effect == None || matches!(&self.effect, Some(eff) if eff.kind != EffectStyle::Damage) {
                                state_changes.push(StateChange::new(*obj_id, col.pos, None));
                            }
                        },
                        ObjectRef::Player(_) => {
                            self.is_alive = false;
                            new_effect = Some(Effect::new(1, EffectStyle::Damage, None, EffectZone::All))
                        }
                    }
                }
            }
        }

        // Add new effect
        if let Some(effect) = new_effect {
            self.effect = Some(effect);

            if let Some(size) = effect.action_size {
                self.resize_head_tmp(size);
            } else if None == effect.action_size {
                self.resize_head_restore();
            }
        }

        if let Some(effect) = self.effect {
            if let Some(size) = effect.action_size {
                self.resize_head_tmp(size);
            }
        }

        // Move head
        state_changes.extend(self.slither_head());

        // Move body

        return Some(state_changes);
    }
}