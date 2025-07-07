mod animation;

use std::collections::HashMap;
use std::{collections::VecDeque};
use crossterm::style::Color;

use crate::game::player::snake::animation::EffectZone;
use crate::game::{food};
use crate::game::grid::{CellKind, ObjectRef};
use crate::game::object::{Collision, DynamicObject, Element, Glyph, Id, IdCounter, Object, Position, ResizeState, StateChange, BodySegment, Orientation};
use animation::{Effect, EffectStyle};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn get_move(&self) -> (i16, i16) {
        let (dx, dy) = match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        return (dx, dy);
    }

    pub fn get_move_reverse(&self) -> (i16, i16) {
        let (dx, dy) = match self {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Left => (1, 0),
            Direction::Right => (-1, 0),
        };
        return (dx, dy);
    }
}

#[derive(Debug)]
pub struct Snake {
    id: Id,
    id_counter: IdCounter, // Id counter for elements (internal use only)
    head_width: ResizeState,
    effect: Option<Effect>,
    is_alive: bool,
    meals: i16,
    head: Vec<Element>,
    body: VecDeque<BodySegment>,
    head_style: Glyph,
    body_style: Glyph,
    direction: Direction,
}

impl Snake {
    pub fn new(pos: Position, obj_id: Id, size: usize) -> Self {
        let head_style = Glyph { fg_clr: Some(Color::DarkMagenta), bg_clr: None, symbol: '█' };
        let body_style = Glyph { fg_clr: Some(Color::DarkYellow), bg_clr: None, symbol: 'S' };
        let mut id_counter = IdCounter::new();
        let first_id = id_counter.next();

        let mut snake = Snake {
            id: obj_id,
            id_counter: id_counter,
            head_width: ResizeState::Normal { size: 1 },
            effect: None,
            is_alive: true,
            meals: 1,
            head: Vec::from([Element::new(first_id, head_style, Some(pos))]),
            body: VecDeque::new(),
            head_style,
            body_style,
            direction: Direction::Down,
        };

        snake.resize_head(size);
        snake
    }

    fn get_head_at(&self, pos: Position) -> Option<&Element> {
        let width = self.head_width.size();
        let pos = pos.y as usize * width + pos.x as usize;
        self.head.get(pos)
    }

    fn resize_head(&mut self, new_size: usize) {
        if let ResizeState::Normal { size } = self.head_width && size == new_size {
            return;
        } else if let ResizeState::Brief { size, .. } = self.head_width && size == new_size {
            self.head_width = ResizeState::Normal { size };
            return;
        }

        let resize = match self.set_size(new_size, true) {
            Some(elements) => elements,
            None => return
        };

        self.head.clear();
        self.head = resize;

        self.head_width = ResizeState::Normal { size: new_size }
    }

    fn resize_head_brief(&mut self, new_size: usize) {
        if self.head_width.size() == new_size {
            return;
        }
        
        let resize = match self.set_size(new_size, true) {
            Some(elements) => elements,
            None => return
        };
    
        self.head.clear();
        self.head = resize;
        
        self.head_width = ResizeState::Brief { size: new_size, native_size: self.head_width.native() };
    }

    fn resize_head_native(&mut self) {
        if let ResizeState::Brief { native_size, .. } = self.head_width {
            let resize = match self.set_size(native_size, true) {
                Some(elements) => elements,
                None => return,
            };

            self.head.clear();
            self.head = resize;
            self.head_width = ResizeState::Normal { size: native_size };
        }
    }

    // TODO - DELETE THIS
    fn get_resized_body_part(&mut self, new_size: usize) -> Option<Vec<Element>> {
        self.set_size(new_size, false)
    }

    fn set_size(&mut self, new_size: usize, is_head: bool) -> Option<Vec<Element>> {
        if self.head.is_empty() {
            return None;
        }

        let odd_size = if new_size % 2 == 0 { new_size.saturating_sub(1).max(1) } else { new_size };

        let center_pos = {
            // Calculate physical boundaries
            let mut min_x = u16::MAX;
            let mut max_x = u16::MIN;
            let mut min_y = u16::MAX;
            let mut max_y = u16::MIN;

            for element in &self.head {
                min_x = min_x.min(element.pos.x);
                max_x = max_x.max(element.pos.x);
                min_y = min_y.min(element.pos.y);
                max_y = max_y.max(element.pos.y);
            }

            Position {
                x: min_x + (max_x - min_x) / 2,
                y: min_y + (max_y - min_y) / 2,
            }
        };

        let new_buttom_left = Position {
            x: center_pos.x.saturating_sub(odd_size as u16 / 2),
            y: center_pos.y.saturating_sub(odd_size as u16 / 2)
        };

        // Operations for head/body
        let style = if is_head { self.head_style } else { self.body_style };
        let rows_iter = if is_head { odd_size } else { 1 };

        // Generates new elements
        let mut new_elements: Vec<Element> = Vec::new();
        for row in 0..rows_iter {
            for col in 0..odd_size {
                let curr_pos = Position {
                    x: new_buttom_left.x + col as u16,
                    y: new_buttom_left.y - row as u16,
                };
                new_elements.push(Element::new(self.id_counter.next(), style, Some(curr_pos)));
            }
        }
        Some(new_elements)
    }

    fn slither (&mut self) -> Vec<StateChange> {
        let mut changes: Vec<StateChange> = Vec::new();

        // Calculate physical boundaries
        let mut min_x = u16::MAX;
        let mut max_x = u16::MIN;
        let mut min_y = u16::MAX;
        let mut max_y = u16::MIN;

        for element in &self.head {
            min_x = min_x.min(element.pos.x);
            max_x = max_x.max(element.pos.x);
            min_y = min_y.min(element.pos.y);
            max_y = max_y.max(element.pos.y);
        }

        let (dx, dy) = self.direction.get_move();
        let mut shed_slice: Vec<&Element> = Vec::new();
        let mut orientation: Orientation;

        match self.direction {
            Direction::Up => {
                orientation = Orientation::Horizontal;
                let iter = max_x - min_x;
                
            },
            Direction::Down => {
                orientation = Orientation::Horizontal;

            },
            Direction::Left => {
                orientation = Orientation::Vertical;

            },
            Direction::Right => {
                orientation = Orientation::Vertical;

            },
        }



        // let mut new_head_row = if let Some(back_row) = self.head.front() {
        //     back_row.clone() } else { return changes; };

        // // Update with new positions and id's
        // for e in new_head_row.iter_mut() {
        //     e.id = self.id_counter.next();
        //     e.pos = Position::new((e.pos.x as i16 + dx) as u16, (e.pos.y as i16 + dy) as u16);
        //     changes.push(StateChange::new(self.id, None, Some(*e)));
        // }

        // self.head.push_front(new_head_row);

        // let mut new_body_row = if let Some(old_head_row) = self.head.pop_back() {
        //     old_head_row } else { return changes; };

        // // Change style to body
        // for e in new_body_row.iter_mut() {
        //     e.style = self.body_style;
        //     changes.push(StateChange::new(self.id, Some(e.pos), Some(*e)));
        // }

        // self.body.push_front(new_body_row);

        changes
    }

    // TODO! - UPDATE TO USE ELEMENT IDS
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
    fn id(&self) -> Id {
        self.id
    }

    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_> {
        Box::new(
            self.head.iter()
            .chain(self.body.iter().flat_map(|segment| &segment.elements))
        )
    }

    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_> {
        Box::new(
            self.head.iter().map(|e| e.pos)
            .chain(self.body.iter().flat_map(|segment| segment.elements.iter().map(|elem| elem.pos)))
        )
    }
}

impl DynamicObject for Snake {
    fn next_pos(&self) -> Box<dyn Iterator<Item = Position> + '_> {
        let (dx, dy) = self.direction.get_move();

        Box::new(self.head.iter().map(move |elem| Position {
            x: (elem.pos.x as i16 + dx) as u16,
            y: (elem.pos.y as i16 + dy) as u16,
        }))
    }

    // TODO! - UPDATE TO USE ELEMENT IDS (HASHMAP)
    fn update(&mut self, collisions: Option<Vec<Collision>>) -> Option<Vec<StateChange>> {
        if !self.is_alive {
            return None;
        }

        let mut state_changes: Vec<StateChange> = Vec::new();
        let mut hashmappy: HashMap<Position, StateChange> = HashMap::new();
        let mut new_effect: Option<Effect> = None;

        // Collision
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
                                food::Kind::Bomb => new_effect = Some(Effect::new(2, EffectStyle::Damage, Some(self.head_width.size() + 2), EffectZone::All)),
                                food::Kind::Cherry => new_effect = Some(Effect::new(2, EffectStyle::Grow, Some(self.head_width.size() + 2), EffectZone::Body)),
                                food::Kind::Mouse => new_effect = Some(Effect::new(2, EffectStyle::Grow, Some(self.head_width.size() + 2), EffectZone::Body)),
                                food::Kind::Grower => { self.resize_head(self.head_width.size() + 2); },
                            }
                            self.meals += meals;

                            state_changes.push(StateChange::new(*obj_id, Some(col.pos), None));
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
        state_changes.extend(self.slither()); // TODO - How to effectively apply EffectZone and EffectStyle for head
        state_changes.extend(self.tick_effect()); // Apply effects

        return Some(state_changes);
    }
}