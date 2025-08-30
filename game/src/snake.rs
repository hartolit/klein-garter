pub mod animation;

use crossterm::style::Color;
use std::hash::Hash;
use std::{collections::VecDeque};

use engine::prelude::*;

use super::game_object::{BodySegment, Orientation, ResizeState};
use animation::{Effect, EffectStyle, EffectZone};

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
    id_counter: IdCounter, // For t_cell ids (internal)
    pub head_size: ResizeState,
    effect: Option<Effect>,
    pub is_alive: bool,
    pub meals: i16,
    head: Vec<TCell>, // Unsorted 2d vec
    body: VecDeque<BodySegment>,
    head_style: Glyph,
    body_style: Glyph,
    state: State,
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
            meals: 10,
            head: Vec::from([TCell::new(
                Occupant::new(obj_id, first_id),
                head_style,
                Some(pos),
            )]),
            body: VecDeque::new(),
            head_style,
            body_style,
            state: State::new(),
            direction: Direction::Down,
        };

        snake.resize_head(size);
        snake
    }

    // TODO! - SIMPLIFY
    pub fn resize_head(&mut self, new_size: usize) {
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

    pub fn resize_head_brief(&mut self, new_size: usize) {
        if self.head_size.size() == new_size {
            return;
        }

        self.set_head_size(new_size);
        self.head_size = ResizeState::Brief {
            size: new_size,
            native_size: self.head_size.native(),
        };
    }

    pub fn resize_head_native(&mut self) {
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

            for t_cell in self.head.iter_mut() {
                min_x = min_x.min(t_cell.pos.x);
                max_x = max_x.max(t_cell.pos.x);
                min_y = min_y.min(t_cell.pos.y);
                max_y = max_y.max(t_cell.pos.y);

                let delete = StateChange::Delete {
                    occupant: t_cell.occ,
                    init_pos: t_cell.pos,
                };
                self.state.upsert_change(delete);
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

        // Generates new t_cells
        self.head.clear();
        for row in 0..odd_size {
            for col in 0..odd_size {
                let curr_pos = Position {
                    x: new_buttom_left.x + col as u16,
                    y: new_buttom_left.y.saturating_sub(row as u16),
                };

                let t_cell = TCell::new(
                    Occupant::new(self.id, self.id_counter.next()),
                    self.head_style,
                    Some(curr_pos),
                );
                let create = StateChange::Create { new_t_cell: t_cell };
                self.state.upsert_change(create);

                self.head.push(t_cell);
            }
        }
    }

    // TODO - CHANGE THIS
    // fn get_resized_body_part(&mut self, new_size: usize) -> Option<Vec<TCell>> {
    //     self.set_head_size(new_size)
    // }

    // TODO! - SIMPLIFY
    fn slither(&mut self) {
        let mut min_x = u16::MAX;
        let mut max_x = u16::MIN;
        let mut min_y = u16::MAX;
        let mut max_y = u16::MIN;

        for t_cell in &self.head {
            min_x = min_x.min(t_cell.pos.x);
            max_x = max_x.max(t_cell.pos.x);
            min_y = min_y.min(t_cell.pos.y);
            max_y = max_y.max(t_cell.pos.y);
        }

        let (dx, dy) = self.direction.get_move();

        let orientation: Orientation;
        let mut new_body: Vec<TCell> = Vec::new();

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

                let new_pos_y = min_y.saturating_add_signed(dy);
                let head_width = (max_x - min_x) + 1;

                for i in 0..head_width {
                    let new_pos = Position::new(min_x + i, new_pos_y);
                    let new_t_cell = TCell::new(
                        Occupant::new(self.id, self.id_counter.next()),
                        self.head_style,
                        Some(new_pos),
                    );

                    let create = StateChange::Create { new_t_cell };
                    self.state.upsert_change(create);
                    self.head.push(new_t_cell);
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

                let new_pos_y = max_y.saturating_add_signed(dy);
                let head_width = (max_x - min_x) + 1;

                for i in 0..head_width {
                    let new_pos = Position::new(min_x + i, new_pos_y);
                    let new_t_cell = TCell::new(
                        Occupant::new(self.id, self.id_counter.next()),
                        self.head_style,
                        Some(new_pos),
                    );

                    let create = StateChange::Create { new_t_cell };
                    self.state.upsert_change(create);

                    self.head.push(new_t_cell);
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

                let new_pos_x = min_x.saturating_add_signed(dx);
                let head_height = (max_y - min_y) + 1;

                for i in 0..head_height {
                    let new_pos = Position::new(new_pos_x, min_y + i);
                    let new_t_cell = TCell::new(
                        Occupant::new(self.id, self.id_counter.next()),
                        self.head_style,
                        Some(new_pos),
                    );

                    let create = StateChange::Create { new_t_cell };
                    self.state.upsert_change(create);

                    self.head.push(new_t_cell);
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

                let new_pos_x = max_x.saturating_add_signed(dx);
                let head_height = (max_y - min_y) + 1;

                for i in 0..head_height {
                    let new_pos = Position::new(new_pos_x, min_y + i);
                    let new_t_cell = TCell::new(
                        Occupant::new(self.id, self.id_counter.next()),
                        self.head_style,
                        Some(new_pos),
                    );

                    let create = StateChange::Create { new_t_cell };
                    self.state.upsert_change(create);

                    self.head.push(new_t_cell);
                }

                orientation = Orientation::Vertical;
            }
        }

        for t_cell in new_body.iter_mut() {
            t_cell.style = self.body_style;

            let update = StateChange::Update {
                t_cell: *t_cell,
                init_pos: t_cell.pos,
            };
            self.state.upsert_change(update);
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
                    for t_cell in segment.t_cells {
                        let delete = StateChange::Delete {
                            occupant: t_cell.occ,
                            init_pos: t_cell.pos,
                        };
                        self.state.upsert_change(delete);
                    }
                }
            }
        } else {
            if let Some(segment) = self.body.pop_back() {
                for t_cell in segment.t_cells {
                    let delete = StateChange::Delete {
                        occupant: t_cell.occ,
                        init_pos: t_cell.pos,
                    };
                    self.state.upsert_change(delete);
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

    pub fn get_t_cells(&self) -> Box<dyn Iterator<Item = &TCell> + '_> {
        Box::new(
            self.head
                .iter()
                .chain(self.body.iter().flat_map(|segment| segment.t_cells.iter())),
        )
    }
}

define_object! {
    struct Snake,
    id_field: id,
    t_cells: custom(get_t_cells),
    capabilities: {
        Stateful { state_field: state }
        Destructible {}
        Movable {
            impl {
                fn predict_pos(&self) -> Box<dyn Iterator<Item = Position> + '_> {
                    let (dx, dy) = self.direction.get_move();

                    Box::new(self.head.iter().map(move |t_cell| Position {
                        x: t_cell.pos.x.saturating_add_signed(dx),
                        y: t_cell.pos.y.saturating_add_signed(dy),
                    }))
                }

                fn make_move(&mut self, probe: Vec<CellRef>) -> Vec<Action> {
                    let actions: Vec<Action> = Vec::new();
                    self.state.changes.clear();
                    if !self.is_alive {
                        return actions;
                    }

                    let mut new_effect: Option<Effect> = None;

                    for hit in probe {
                        if let Kind::Border | Kind::Lava = hit.cell.kind {
                            //self.is_alive = false;
                            new_effect = Some(Effect::new(1, EffectStyle::Damage, None, EffectZone::All))
                        }

                        // let hit_object = match game_objects.get(&hit.cell.occ_by.obj_id) {
                        //     Some(object) => object,
                        //     None => continue,
                        // };

                        // if let Some(consumable) = hit_object.as_consumable() {
                        //     self.meals += consumable.get_meal();
                        //     let change = consumable.on_consumed(hit.cell.occ_by.element_id, hit.pos, self.id);
                        //     new_effect = Some(Effect::new(
                        //         2,
                        //         EffectStyle::Grow,
                        //         Some(self.head_size.size() + 2),
                        //         EffectZone::All,
                        //     ));
                        //     self.state_manager.upsert_change(change);
                        // }

                        // if let Some(damaging) = hit_object.as_damaging() {
                        //     self.meals += damaging.get_damage();
                        //     new_effect = Some(Effect::new(2, EffectStyle::Damage, None, EffectZone::All));
                        // }

                        // if let Some(_) = hit_object.get::<Snake>() {
                        //     self.is_dead = false;
                        // }
                    }

                    self.slither();
                    if let Some(effect) = new_effect {
                        self.apply_effect(effect);
                    }

                    self.tick_effect();

                    actions
                }
            }
        }
    }
}