mod animation;

use crossterm::style::Color;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::hash::Hash;

use crate::game::food;
use crate::game::grid::{CellKind, ObjectRef};
use crate::game::object::{
    BodySegment, Collision, DynamicObject, Element, Glyph, Id, IdCounter, Object, Orientation,
    Position, ResizeState, StateChange,
};
use crate::game::player::snake::animation::EffectZone;
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
}

#[derive(Debug)]
pub struct Snake {
    id: Id,
    id_counter: IdCounter, // For element ids (internal)
    head_size: ResizeState,
    effect: Option<Effect>,
    is_alive: bool,
    meals: i16,
    head: Vec<Element>, // Unsorted 2d vec
    body: VecDeque<BodySegment>,
    head_style: Glyph,
    body_style: Glyph,
    changes: HashMap<(Id, Id), StateChange>,
    pub direction: Direction,
}

impl Snake {
    pub fn new(pos: Position, obj_id: Id, size: usize) -> Self {
        let head_style = Glyph {
            fg_clr: Some(Color::DarkMagenta),
            bg_clr: None,
            symbol: '█',
        };
        let body_style = Glyph {
            fg_clr: Some(Color::DarkYellow),
            bg_clr: None,
            symbol: 'S',
        };
        let mut id_counter = IdCounter::new();
        let first_id = id_counter.next();

        let mut snake = Snake {
            id: obj_id,
            id_counter: id_counter,
            head_size: ResizeState::Normal { size: 1 },
            effect: None,
            is_alive: true,
            meals: 1,
            head: Vec::from([Element::new(first_id, head_style, Some(pos))]),
            body: VecDeque::new(),
            head_style,
            body_style,
            changes: HashMap::new(),
            direction: Direction::Down,
        };

        snake.resize_head(size);
        snake
    }

    fn upsert_change(changes: &mut HashMap<(Id, Id), StateChange>, new_state: StateChange) {
        let key = match new_state {
            StateChange::Create { obj_id, element_id, .. } => (obj_id, element_id),
            StateChange::Update { obj_id, element_id, .. } => (obj_id, element_id),
            StateChange::Delete { obj_id, element_id, .. } => (obj_id, element_id),
        };

        match changes.entry(key) {
            Entry::Occupied(mut entry) => {
                let curr_state = entry.get_mut();

                match curr_state {
                    StateChange::Create { new_element: curr_element, .. } => {
                        match new_state {
                            StateChange::Create { new_element, .. } => {
                                *curr_element = new_element;
                            },
                            StateChange::Update { element, .. } => {
                                *curr_element = element;
                            },
                            StateChange::Delete { .. } => {
                                entry.remove();
                            }
                        }
                    },

                    StateChange::Update { element: curr_element, old_pos: curr_old_pos, .. } => {
                        match new_state {
                            StateChange::Create { new_element, .. } => {
                                *curr_element = new_element;
                            },
                            StateChange::Update { element, .. } => {
                                *curr_element = element;
                            },
                            StateChange::Delete { obj_id, element_id , .. } => {
                                *curr_state = StateChange::Delete { obj_id, element_id, old_pos: *curr_old_pos };
                            }
                        }
                    }

                    StateChange::Delete { .. } => { }
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(new_state);
            },
        }
    }

    // TODO! - SIMPLIFY
    fn resize_head(&mut self, new_size: usize) {
        if let ResizeState::Normal { size } = self.head_size
            && size == new_size
        {
            return;
        } else if let ResizeState::Brief { size, .. } = self.head_size
            && size == new_size
        {
            self.head_size = ResizeState::Normal { size };
            return;
        }
        self.set_head_size(new_size);
        self.head_size = ResizeState::Normal { size: new_size };
    }

    fn resize_head_brief(&mut self, new_size: usize) {
        if self.head_size.size() == new_size {
            return;
        }

        self.set_head_size(new_size);
        self.head_size = ResizeState::Brief {
            size: new_size,
            native_size: self.head_size.native(),
        };
    }

    fn resize_head_native(&mut self) {
        if let ResizeState::Brief { native_size, .. } = self.head_size {
            self.set_head_size(native_size);
            self.head_size = ResizeState::Normal { size: native_size };
        }
    }

    // TODO! - SIMPLIFY
    fn set_head_size(&mut self, new_size: usize) {
        if self.head.is_empty() {
            return;
        }

        let odd_size = if new_size % 2 == 0 {
            new_size.saturating_sub(1).max(1)
        } else {
            new_size
        };

        let center_pos = {
            // Calculate physical boundaries
            let mut min_x = u16::MAX;
            let mut max_x = u16::MIN;
            let mut min_y = u16::MAX;
            let mut max_y = u16::MIN;

            for element in self.head.iter_mut() {
                min_x = min_x.min(element.pos.x);
                max_x = max_x.max(element.pos.x);
                min_y = min_y.min(element.pos.y);
                max_y = max_y.max(element.pos.y);
                Self::upsert_change(&mut self.changes, StateChange::new(self.id, Some(element.pos), None), element.id);
            }

            Position {
                x: min_x + (max_x - min_x) / 2,
                y: min_y + (max_y - min_y) / 2,
            }
        };

        let new_buttom_left = Position {
            x: center_pos.x.saturating_sub(odd_size as u16 / 2),
            y: center_pos.y.saturating_sub(odd_size as u16 / 2),
        };

        // Generates new elements
        self.head.clear();
        for row in 0..odd_size {
            for col in 0..odd_size {
                let curr_pos = Position {
                    x: new_buttom_left.x + col as u16,
                    y: new_buttom_left.y - row as u16,
                };

                let element = Element::new(self.id_counter.next(), self.head_style, Some(curr_pos));
                Self::upsert_change(&mut self.changes, StateChange::new(self.id, None, Some(element)), element.id);
                self.head.push(element);
            }
        }
    }

    // TODO - CHANGE THIS
    // fn get_resized_body_part(&mut self, new_size: usize) -> Option<Vec<Element>> {
    //     self.set_head_size(new_size)
    // }

    // TODO! - SIMPLIFY
    fn slither(&mut self) {
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

        let orientation: Orientation;
        let mut new_body: Vec<Element> = Vec::new();

        // TODO - use drain_filter when stable
        // shed_slice = self.head.drain_filter(|e| e.pos.x == max_x).collect();
        match self.direction {
            Direction::Up => {
                self.head.retain(|e| {
                    if e.pos.y == max_y {
                        new_body.push(*e);
                        false
                    } else {
                        true
                    }
                });

                let new_pos_y = min_y + dy as u16;
                let head_width = (max_x - min_x) + 1;

                for i in 0..head_width {
                    let new_pos = Position::new(min_x + i, new_pos_y);
                    let new_element =
                        Element::new(self.id_counter.next(), self.head_style, Some(new_pos));
                    Self::upsert_change(&mut self.changes, StateChange::new(self.id, None, Some(new_element)), new_element.id);
                    self.head.push(new_element);
                }

                orientation = Orientation::Horizontal;
            }
            Direction::Down => {
                self.head.retain(|e| {
                    if e.pos.y == min_y {
                        new_body.push(*e);
                        false
                    } else {
                        true
                    }
                });

                let new_pos_y = max_y + dy as u16;
                let head_width = (max_x - min_x) + 1;

                for i in 0..head_width {
                    let new_pos = Position::new(min_x + i, new_pos_y);
                    let new_element =
                        Element::new(self.id_counter.next(), self.head_style, Some(new_pos));
                    Self::upsert_change(&mut self.changes, StateChange::new(self.id, None, Some(new_element)), new_element.id);
                    self.head.push(new_element);
                }

                orientation = Orientation::Horizontal;
            }
            Direction::Left => {
                self.head.retain(|e| {
                    if e.pos.x == max_x {
                        new_body.push(*e);
                        false
                    } else {
                        true
                    }
                });

                let new_pos_x = min_x + dx as u16;
                let head_height = (max_y - min_y) + 1;

                for i in 0..head_height {
                    let new_pos = Position::new(new_pos_x, min_y + i);
                    let new_element =
                        Element::new(self.id_counter.next(), self.head_style, Some(new_pos));
                    Self::upsert_change(&mut self.changes, StateChange::new(self.id, None, Some(new_element)), new_element.id);
                    self.head.push(new_element);
                }

                orientation = Orientation::Vertical;
            }
            Direction::Right => {
                self.head.retain(|e| {
                    if e.pos.x == min_x {
                        new_body.push(*e);
                        false
                    } else {
                        true
                    }
                });

                let new_pos_x = max_x + dx as u16;
                let head_height = (max_y - min_y) + 1;

                for i in 0..head_height {
                    let new_pos = Position::new(new_pos_x, min_y + i);
                    let new_element =
                        Element::new(self.id_counter.next(), self.head_style, Some(new_pos));
                    Self::upsert_change(&mut self.changes, StateChange::new(self.id, None, Some(new_element)), new_element.id);
                    self.head.push(new_element);
                }

                orientation = Orientation::Vertical;
            }
        }

        for element in new_body.iter_mut() {
            element.style = self.body_style;
            Self::upsert_change(&mut self.changes, StateChange::new(self.id, Some(element.pos), Some(*element)), element.id);
        }

        self.body
            .push_front(BodySegment::new(orientation, new_body));

        if self.meals > 0 {
            self.meals -= 1;
        } else if self.meals < 0 {
            let segments_to_remove = self.meals.abs() as u16;
            for _ in 0..segments_to_remove {
                if self.body.len() == 0 {
                    self.is_alive = false;
                    break;
                }
                if let Some(segment) = self.body.pop_back() {
                    for element in segment.elements {
                        Self::upsert_change(&mut self.changes, StateChange::new(self.id, Some(element.pos), None), element.id);
                    }
                }
            }
        } else {
            if let Some(segment) = self.body.pop_back() {
                for element in segment.elements {
                    Self::upsert_change(&mut self.changes, StateChange::new(self.id, Some(element.pos), None), element.id);
                }
            }
        }
    }

    // TODO - implement EffectZone and EffectStyle
    fn tick_effect(&mut self) {

        let Some(mut effect) = self.effect.take() else {
            return;
        };

        effect.next_tick();

        if effect.is_expired() {
            self.resize_head_native();
            self.effect = None;
        } else {
            self.effect = Some(effect)
        }
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
            self.head
                .iter()
                .chain(self.body.iter().flat_map(|segment| &segment.elements)),
        )
    }

    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_> {
        Box::new(
            self.head.iter().map(|e| e.pos).chain(
                self.body
                    .iter()
                    .flat_map(|segment| segment.elements.iter().map(|elem| elem.pos)),
            ),
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

    fn update(
        &mut self,
        collisions: Option<Vec<Collision>>,
    ) -> Option<HashMap<(Id, Id), StateChange>> {
        if !self.is_alive {
            return None;
        }

        self.changes.clear();

        let mut new_effect: Option<Effect> = None;
        if let Some(cols) = collisions {
            for col in cols {
                if let CellKind::Border | CellKind::Lava = col.kind {
                    self.is_alive = false;
                    new_effect = Some(Effect::new(1, EffectStyle::Damage, None, EffectZone::All))
                }

                for obj_ref in col.colliders {
                    match obj_ref {
                        ObjectRef::Food {
                            obj_id,
                            kind,
                            meals,
                            elem_id,
                        } => {
                            match &*kind {
                                food::Kind::Bomb => {
                                    new_effect = Some(Effect::new(
                                        2,
                                        EffectStyle::Damage,
                                        Some(self.head_size.size() + 2),
                                        EffectZone::All,
                                    ))
                                }
                                food::Kind::Cherry => {
                                    new_effect = Some(Effect::new(
                                        2,
                                        EffectStyle::Grow,
                                        Some(self.head_size.size() + 2),
                                        EffectZone::Body,
                                    ))
                                }
                                food::Kind::Mouse => {
                                    new_effect = Some(Effect::new(
                                        2,
                                        EffectStyle::Grow,
                                        Some(self.head_size.size() + 2),
                                        EffectZone::Body,
                                    ))
                                }
                                food::Kind::Grower => {
                                    self.resize_head(self.head_size.size() + 2);
                                }
                            }
                            self.meals += meals;

                            Self::upsert_change(&mut self.changes, StateChange::new(*obj_id, Some(col.pos), None), *elem_id);
                        }
                        ObjectRef::Player(_) => {
                            self.is_alive = false;
                            new_effect =
                                Some(Effect::new(1, EffectStyle::Damage, None, EffectZone::All))
                        }
                    }
                }
            }
        }

        self.slither();
        if let Some(effect) = new_effect {
            self.apply_effect(effect);
        }
        self.tick_effect();

        if self.changes.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.changes))
        }
    }
}
