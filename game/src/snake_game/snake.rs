pub mod animation;

use crossterm::style::Color;
use std::collections::VecDeque;
use std::hash::Hash;

use engine::prelude::*;

use super::events::{CollisionEvent, DeathEvent};
use super::game_object::{BodySegment, Orientation, ResizeState};
use animation::Effect;

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
    pub head_style: Glyph,
    pub body_style: Glyph,
    state: State,
    pub direction: Direction,
    pub base_index: u8,
    pub ignore_all: bool,
    pub ignore_body: bool,
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
            ignore_all: false,
            ignore_body: false,
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
        if self.head_size.current_size() == new_size {
            return;
        }

        self.set_head_size(new_size);
        self.head_size = ResizeState::Brief {
            size: new_size,
            native_size: self.head_size.native_size(),
        };
    }

    pub fn resize_head_native(&mut self) {
        if let ResizeState::Brief { native_size, .. } = self.head_size {
            self.set_head_size(native_size);
            self.head_size = ResizeState::Normal { size: native_size };
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
            self.resize_head_native();
            self.effect = None;
        } else {
            self.effect = Some(effect)
        }
    }

    pub fn apply_effect(&mut self, new_effect: Effect) {
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
        Spatial {}
        Movable {
            impl {
                fn predict_pos(&self) -> Box<dyn Iterator<Item = Position> + '_> {
                    if self.head.is_empty() {
                        return Box::new(std::iter::empty());
                    }

                    let (dx, dy) = self.direction.get_move();

                    let (min_x, max_x, min_y, max_y) = self.head.iter().fold(
                    (u16::MAX, u16::MIN, u16::MAX, u16::MIN),
                    |(min_x, max_x, min_y, max_y), t_cell| {
                        (
                            min_x.min(t_cell.pos.x),
                            max_x.max(t_cell.pos.x),
                            min_y.min(t_cell.pos.y),
                            max_y.max(t_cell.pos.y),
                        )
                    });

                    // Filter for only the "leading edge" cells based on direction
                    let leading_edge = self.head.iter().filter(move |t_cell| match self.direction {
                        Direction::Up => t_cell.pos.y == min_y,
                        Direction::Down => t_cell.pos.y == max_y,
                        Direction::Left => t_cell.pos.x == min_x,
                        Direction::Right => t_cell.pos.x == max_x,
                    });

                    // Predict the next position for only those leading cells
                    Box::new(leading_edge.map(move |t_cell| Position {
                        x: t_cell.pos.x.saturating_add_signed(dx),
                        y: t_cell.pos.y.saturating_add_signed(dy),
                    }))
                }

                fn make_move(&mut self, probe: Vec<CellRef>) -> Vec<Box<dyn Event>> {
                    let mut events: Vec<Box<dyn Event>> = Vec::new();

                    if !self.ignore_all {
                        for hit in probe {
                            if let Some(t_cell) = hit.cell.occ_by {
                                if t_cell.occ.obj_id == self.id {
                                    if self.ignore_body {
                                        continue;
                                    }
                                    
                                    let event = DeathEvent {
                                        actor: self.id,
                                        pos: hit.pos,
                                    };
                                    events.push(Box::new(event));
                                    return events;
                                }

                                let event = CollisionEvent {
                                        actor: self.id,
                                        target: t_cell.occ.obj_id,
                                        pos: hit.pos,
                                    };
                                events.push(Box::new(event));
                            }
                        }
                    }

                    self.slither();
                    self.tick_effect();

                    events
                }
            }
        }
    }
}
