pub mod animation;
pub mod utils;

use crossterm::style::Color;
use std::collections::VecDeque;

use engine::prelude::*;

use crate::snake_game::events::{CollisionEvent, DeathEvent};
use animation::Effect;
pub use utils::{BodySegment, Direction, Orientation, ResizeState};

#[derive(Debug)]
pub struct Snake {
    id: Id,
    id_counter: IdCounter, // For t_cell ids (internal)
    pub head_size: ResizeState,
    pending_resize: Option<ResizeState>,
    effect: Option<Effect>,
    pub is_alive: bool,
    pub meals: i16,
    head: Vec<TCell>, // Unsorted 2d vec
    body: VecDeque<BodySegment>,
    pub head_style: Glyph,
    pub body_style: Glyph,
    state: State,
    pub direction: Direction,
    pub base_index: u8,
    pub ignore_death: bool,
    pub ignore_body: bool,
    pub is_moving: bool,
}

impl Snake {
    pub fn new(pos: Position, obj_id: Id, size: usize) -> Self {
        let head_style = Glyph {
            fg_clr: Some(Color::Cyan),
            bg_clr: Some(Color::Black),
            symbol: '1',
        };
        let body_style = Glyph {
            fg_clr: Some(Color::DarkBlue),
            bg_clr: Some(Color::Black),
            symbol: '0',
        };
        let mut id_counter = IdCounter::new();
        let first_id = id_counter.next();

        let base_index = 10;

        let mut snake = Snake {
            id: obj_id,
            id_counter: id_counter,
            head_size: ResizeState::Normal { size: 1 },
            pending_resize: None,
            effect: None,
            is_alive: true,
            meals: 30,
            head: Vec::from([TCell::new(
                Occupant::new(obj_id, first_id),
                head_style,
                Some(pos),
                base_index,
            )]),
            body: VecDeque::new(),
            head_style,
            body_style,
            state: State::new(),
            direction: Direction::Down,
            base_index,
            ignore_death: false,
            ignore_body: false,
            is_moving: true,
        };

        snake.resize_head_native(size);

        snake
    }

    pub fn resize_head_native(&mut self, new_size: usize) {
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

        self.pending_resize = Some(ResizeState::Normal { size: new_size });
    }

    pub fn resize_head_brief(&mut self, new_size: usize) {
        if self.head_size.current_size() == new_size {
            return;
        }

        self.pending_resize = Some(ResizeState::Brief {
            size: new_size,
            native_size: self.head_size.native_size(),
        });
    }

    pub fn reset_head_size(&mut self) {
        if let ResizeState::Brief { native_size, .. } = self.head_size {
            self.pending_resize = Some(ResizeState::Normal { size: native_size });
        }
    }

    fn set_head_size(&mut self, new_size: usize) {
        if self.head.is_empty() {
            return;
        }

        let odd_size = if new_size % 2 == 0 {
            new_size.saturating_sub(1).max(1)
        } else {
            new_size
        };

        let (min_x, max_x, min_y, max_y) = self.head.iter().fold(
            (u16::MAX, u16::MIN, u16::MAX, u16::MIN),
            |(min_x, max_x, min_y, max_y), t_cell| {
                // Delete old head state
                let delete = StateChange::Delete {
                    occupant: t_cell.occ,
                    init_pos: t_cell.pos,
                };
                self.state.upsert_change(delete);
                (
                    min_x.min(t_cell.pos.x),
                    max_x.max(t_cell.pos.x),
                    min_y.min(t_cell.pos.y),
                    max_y.max(t_cell.pos.y),
                )
            },
        );

        self.head.clear();

        let half_size = odd_size as u16 / 2;
        let center_x = min_x + (max_x - min_x) / 2;
        let center_y = min_y + (max_y - min_y) / 2;

        // Determines the top-left corner of the new head based on the snake's direction.
        // This anchors the resize to the "back" of the head, preventing an overlap with the body.
        let top_left = match self.direction {
            Direction::Up => Position {
                x: center_x.saturating_sub(half_size),
                y: max_y.saturating_sub(odd_size as u16 - 1),
            },
            Direction::Down => Position {
                x: center_x.saturating_sub(half_size),
                y: min_y,
            },
            Direction::Left => Position {
                x: max_x.saturating_sub(odd_size as u16 - 1),
                y: center_y.saturating_sub(half_size),
            },
            Direction::Right => Position {
                x: min_x,
                y: center_y.saturating_sub(half_size),
            },
        };

        // Generates the new head
        for row in 0..odd_size {
            for col in 0..odd_size {
                let curr_pos = Position {
                    x: top_left.x + col as u16,
                    y: top_left.y + row as u16,
                };

                let t_cell = TCell::new(
                    Occupant::new(self.id, self.id_counter.next()),
                    self.head_style,
                    Some(curr_pos),
                    self.base_index,
                );
                let create = StateChange::Create { new_t_cell: t_cell };
                self.state.upsert_change(create);

                self.head.push(t_cell);
            }
        }
    }

    fn slither(&mut self) {
        let (min_x, max_x, min_y, max_y) = self.head.iter().fold(
            (u16::MAX, u16::MIN, u16::MAX, u16::MIN),
            |(min_x, max_x, min_y, max_y), t_cell| {
                (
                    min_x.min(t_cell.pos.x),
                    max_x.max(t_cell.pos.x),
                    min_y.min(t_cell.pos.y),
                    max_y.max(t_cell.pos.y),
                )
            },
        );

        let (dx, dy) = self.direction.get_move();

        let mut new_body_cells: Vec<TCell> = Vec::new();

        let (new_head_positions, orientation) = match self.direction {
            Direction::Up => {
                self.head.retain(|cell| {
                    if cell.pos.y == max_y {
                        new_body_cells.push(*cell);
                        return false;
                    }
                    true
                });

                let head_width = (max_x - min_x) + 1;
                let new_y = min_y.saturating_add_signed(dy);
                let positions = (0..head_width)
                    .map(|i| Position::new(min_x + i, new_y))
                    .collect::<Vec<_>>();

                (positions, Orientation::Horizontal)
            }
            Direction::Down => {
                self.head.retain(|cell| {
                    if cell.pos.y == min_y {
                        new_body_cells.push(*cell);
                        return false;
                    }
                    true
                });

                let head_width = (max_x - min_x) + 1;
                let new_y = max_y.saturating_add_signed(dy);
                let positions = (0..head_width)
                    .map(|i| Position::new(min_x + i, new_y))
                    .collect::<Vec<_>>();

                (positions, Orientation::Horizontal)
            }
            Direction::Left => {
                self.head.retain(|cell| {
                    if cell.pos.x == max_x {
                        new_body_cells.push(*cell);
                        return false;
                    }
                    true
                });

                let head_height = (max_y - min_y) + 1;
                let new_x = min_x.saturating_add_signed(dx);
                let positions = (0..head_height)
                    .map(|i| Position::new(new_x, min_y + i))
                    .collect::<Vec<_>>();

                (positions, Orientation::Vertical)
            }
            Direction::Right => {
                self.head.retain(|cell| {
                    if cell.pos.x == min_x {
                        new_body_cells.push(*cell);
                        return false;
                    }
                    true
                });

                let head_height = (max_y - min_y) + 1;
                let new_x = max_x.saturating_add_signed(dx);
                let positions = (0..head_height)
                    .map(|i| Position::new(new_x, min_y + i))
                    .collect::<Vec<_>>();

                (positions, Orientation::Vertical)
            }
        };

        let new_head_cells: Vec<TCell> = new_head_positions
            .into_iter()
            .map(|pos| {
                let t_cell = TCell::new(
                    Occupant::new(self.id, self.id_counter.next()),
                    self.head_style,
                    Some(pos),
                    self.base_index,
                );

                // Add new head state
                self.state
                    .upsert_change(StateChange::Create { new_t_cell: t_cell });

                t_cell
            })
            .collect();

        self.head.extend(new_head_cells);

        for t_cell in new_body_cells.iter_mut() {
            t_cell.style = self.body_style;
            self.state.upsert_change(StateChange::Update {
                t_cell: *t_cell,
                init_pos: t_cell.pos,
            });
        }

        self.body
            .push_front(BodySegment::new(orientation, new_body_cells));

        if self.meals > 0 {
            self.meals -= 1;
        } else {
            let segments_to_remove = if self.meals == 0 {
                1
            } else {
                self.meals.abs() as usize
            };
            for _ in 0..segments_to_remove {
                if let Some(segment) = self.body.pop_back() {
                    for t_cell in segment.t_cells {
                        self.state.upsert_change(StateChange::Delete {
                            occupant: t_cell.occ,
                            init_pos: t_cell.pos,
                        });
                    }
                } else {
                    self.is_alive = false;
                    break;
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
            self.reset_head_size();
            self.effect = None;
        } else {
            self.effect = Some(effect)
        }
    }

    pub fn apply_effect(&mut self, new_effect: Effect) {
        if let Some(size) = new_effect.action_size {
            self.resize_head_brief(size);
        } else {
            self.reset_head_size();
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

    // TODO - Fix
    fn predict_head_resize_pos(&self, new_size: usize) -> Vec<Position> {
        if self.head.is_empty() {
            return Vec::new();
        }

        let odd_size = if new_size % 2 == 0 {
            new_size.saturating_sub(1).max(1)
        } else {
            new_size
        };

        let (min_x, max_x, min_y, max_y) = self.head.iter().fold(
            (u16::MAX, u16::MIN, u16::MAX, u16::MIN),
            |(min_x, max_x, min_y, max_y), t_cell| {
                (
                    min_x.min(t_cell.pos.x),
                    max_x.max(t_cell.pos.x),
                    min_y.min(t_cell.pos.y),
                    max_y.max(t_cell.pos.y),
                )
            },
        );

        let half_size = odd_size as u16 / 2;
        let center_x = min_x + (max_x - min_x) / 2;
        let center_y = min_y + (max_y - min_y) / 2;

        let top_left = match self.direction {
            Direction::Up => Position {
                x: center_x.saturating_sub(half_size),
                y: max_y.saturating_sub(odd_size as u16 - 1),
            },
            Direction::Down => Position {
                x: center_x.saturating_sub(half_size),
                y: min_y,
            },
            Direction::Left => Position {
                x: max_x.saturating_sub(odd_size as u16 - 1),
                y: center_y.saturating_sub(half_size),
            },
            Direction::Right => Position {
                x: min_x,
                y: center_y.saturating_sub(half_size),
            },
        };

        let mut positions = Vec::with_capacity(odd_size * odd_size);
        for row in 0..odd_size {
            for col in 0..odd_size {
                positions.push(Position {
                    x: top_left.x + col as u16,
                    y: top_left.y + row as u16,
                });
            }
        }
        positions
    }
}

define_object! {
    struct Snake,
    id_field: id,
    t_cells: custom(get_t_cells),
    capabilities: {
        Stateful { state_field: state }
        Destructible {}
        Spatial {}
        Movable {
            impl {
                fn probe_move(&self) -> Box<dyn Iterator<Item = Position> + '_> {
                    if self.head.is_empty() || !self.is_moving {
                        return Box::new(std::iter::empty());
                    }

                    let (curr_min_x, curr_max_x, curr_min_y, curr_max_y) = self.head.iter().fold(
                            (u16::MAX, u16::MIN, u16::MAX, u16::MIN),
                            |(min_x, max_x, min_y, max_y), t_cell| {
                                (min_x.min(t_cell.pos.x), max_x.max(t_cell.pos.x),
                                min_y.min(t_cell.pos.y), max_y.max(t_cell.pos.y))
                            });


                    let (dx, dy) = self.direction.get_move();

                    if let Some(resize) = self.pending_resize {
                        let new_size = resize.current_size();
                        let is_growing = new_size > self.head_size.current_size();
                        let future_head_pos = self.predict_head_resize_pos(new_size);

                        if is_growing {
                            // Bounding box of the new head size
                            let (new_min_x, new_max_x, new_min_y, new_max_y) = future_head_pos.iter().fold(
                                (u16::MAX, u16::MIN, u16::MAX, u16::MIN),
                                |(min_x, max_x, min_y, max_y), pos| {
                                    (min_x.min(pos.x), max_x.max(pos.x),
                                    min_y.min(pos.y), max_y.max(pos.y))
                                });

                                let future_move = future_head_pos.into_iter().flat_map(move |pos| {
                                    // A tiny array to hold None or pos + a generated pos
                                    let mut positions_to_yield = [None; 2];
                                    let mut index = 0;

                                    let is_leading_edge = match self.direction {
                                        Direction::Up => pos.y == new_min_y,
                                        Direction::Down => pos.y == new_max_y,
                                        Direction::Left => pos.x == new_min_x,
                                        Direction::Right => pos.x == new_max_x,
                                    };

                                    if is_leading_edge {
                                        positions_to_yield[index] = Some(Position {
                                            x: pos.x.saturating_add_signed(dx),
                                            y: pos.y.saturating_add_signed(dy),
                                        });
                                        index += 1;
                                    }

                                    // Expansion check from the current bounding box
                                    let is_expansion = pos.x < curr_min_x
                                    || pos.x > curr_max_x
                                    || pos.y < curr_min_y
                                    || pos.y > curr_max_y;
                                    if is_expansion {
                                        positions_to_yield[index] = Some(pos);
                                    }

                                    // Returns an iterator over the yielded positions.
                                    positions_to_yield.into_iter().flatten()
                                });

                                Box::new(future_move)
                            } else {
                                // Returns a single probe targeting our own head cell.
                                // This is to ensure the engine calls `make_move`.
                                // The resize "grace period" ignores this collision.
                                Box::new(std::iter::once(self.head[0].pos))
                            }
                    } else {
                        // Predicts the next position for only the leading edge
                        let leading_edge = self.head.iter().filter(move |t_cell| match self.direction {
                            Direction::Up => t_cell.pos.y == curr_min_y,
                            Direction::Down => t_cell.pos.y == curr_max_y,
                            Direction::Left => t_cell.pos.x == curr_min_x,
                            Direction::Right => t_cell.pos.x == curr_max_x,
                        });

                        Box::new(leading_edge.map(move |t_cell| Position {
                            x: t_cell.pos.x.saturating_add_signed(dx),
                            y: t_cell.pos.y.saturating_add_signed(dy),
                        }))
                    }
                }

                fn make_move(&mut self, probe: Vec<CellRef>) -> Vec<Box<dyn Event>> {
                    let mut events: Vec<Box<dyn Event>> = Vec::new();

                    for hit in probe {
                        if let Some(t_cell) = hit.cell.occ_by {
                            if t_cell.occ.obj_id == self.id {

                                if self.ignore_body
                                    || self.pending_resize.is_some() // Resize grace period
                                    || self.ignore_death {
                                    continue;
                                }

                                let event = DeathEvent {
                                    actor: self.id,
                                    pos: hit.pos,
                                };
                                events.clear();
                                events.push(Box::new(event));
                                return events;
                            }

                            let event = CollisionEvent {
                                    actor: self.id,
                                    target: t_cell.occ.obj_id,
                                    pos: hit.pos,
                                    ignore: self.ignore_death,
                                };
                            events.push(Box::new(event));
                        }
                    }

                    if let Some(resize) = self.pending_resize {
                        match resize {
                            ResizeState::Brief { size, .. } => {
                                self.set_head_size(size);
                                self.head_size = resize;
                            },
                            ResizeState::Normal { size } => {
                                self.set_head_size(size);
                                self.head_size = resize;
                            },
                        }
                        self.pending_resize = None;
                    }

                    self.slither();
                    self.tick_effect();

                    events
                }
            }
        }
    }
}
