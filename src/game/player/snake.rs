mod animation;

use std::{collections::VecDeque};
use crossterm::style::Color;

use crate::game::player::snake::animation::EffectZone;
use crate::game::{food, State};
use crate::game::grid::{CellKind, ObjectRef};
use crate::game::object::{Collision, DynamicObject, Element, Glyph, Object, ObjectId, Position, StateChange, ResizeState};
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
    resize_state: ResizeState,
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
            resize_state: ResizeState::Normal { size: 1 },
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
        if let ResizeState::Normal { size } = self.resize_state && size == new_size {
            return;
        } else if let ResizeState::Brief { size, .. } = self.resize_state && size == new_size {
            self.resize_state = ResizeState::Normal { size };
            return;
        }

        let resize = self.set_size(new_size, true);

        self.head.clear();
        self.head = resize;

        self.resize_state = ResizeState::Normal { size: new_size }
    }

    fn resize_head_brief(&mut self, new_size: usize) {
        if self.resize_state.size() == new_size {
            return;
        }

        let resize = self.set_size(new_size, true);
    
        self.head.clear();
        self.head = resize;
        
        self.resize_state = ResizeState::Brief { size: new_size, native_size: self.resize_state.native() };
    }

    fn resize_head_native(&mut self) {
        if let ResizeState::Brief { native_size, .. } = self.resize_state {
            let resize = self.set_size(native_size, true);
            self.head.clear();
            self.head = resize;
            self.resize_state = ResizeState::Normal { size: native_size };
        }
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
            let curr_size = self.resize_state.size();

            let center_pos = Position {
                x: tmp_pos.x + curr_size as u16 / 2,
                y: tmp_pos.y + curr_size as u16 / 2,
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

    // TODO - implement slither body (beware of effect implementation with resized body parts for a dynamic effect!)
    fn slither_body (&mut self) -> Vec<StateChange> {
        let mut changes: Vec<StateChange> = Vec::new();

        changes
    }

    // TODO - Return Vec<StateChange> with option faster?
    // TODO - implement EffectZone and EffectStyle
    fn tick_effect(&mut self) -> Vec<StateChange> {
        let mut changes: Vec<StateChange> = Vec::new();

        let Some(mut effect) = self.effect.take() else {
            return changes;
        };

        effect.next_tick();

        if effect.is_expired() {
            self.resize_head_native();
            self.effect = None;
        } else {
            self.effect = Some(effect) // Put it back?
        }

        changes
    }

    fn apply_effect(&mut self, new_effect: Effect) {
        if let Some(size) = new_effect.action_size {
            self.resize_head_brief(size);
        } else {
            self.resize_head_native();
        }
        self.effect = Some(new_effect);
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
                    new_effect = Some(Effect::new(1, EffectStyle::Damage, None, EffectZone::All))
                }

                for obj_ref in col.colliders {
                    match obj_ref {
                        ObjectRef::Food { obj_id, kind, meals } => {
                            match &*kind {
                                food::Kind::Bomb => new_effect = Some(Effect::new(2, EffectStyle::Damage, Some(self.resize_state.size() + 2), EffectZone::All)),
                                food::Kind::Cherry => new_effect = Some(Effect::new(2, EffectStyle::Grow, Some(self.resize_state.size() + 2), EffectZone::Body)),
                                food::Kind::Mouse => new_effect = Some(Effect::new(2, EffectStyle::Grow, Some(self.resize_state.size() + 2), EffectZone::Body)),
                                food::Kind::Grower => { self.resize_head(self.resize_state.size() + 2); },
                            }
                            self.meals += meals;

                            state_changes.push(StateChange::new(*obj_id, col.pos, None));
                        },
                        ObjectRef::Player(_) => {
                            self.is_alive = false;
                            new_effect = Some(Effect::new(1, EffectStyle::Damage, None, EffectZone::All))
                        }
                    }
                }
            }
        }

        if let Some(effect) = new_effect {
            self.apply_effect(effect);
        }

        // TODO - Fix state_changes
        // Effects could apply to the whole snake
        // and potenially save stale data as the
        // snake changes state.
        state_changes.extend(self.tick_effect()); // Apply effects
        state_changes.extend(self.slither_head()); // TODO - How to effectively apply EffectZone and EffectStyle for head
        state_changes.extend(self.slither_body()); // TODO - How to effectively apply EffectZone and EffectStyle for body

        return Some(state_changes);
    }
}