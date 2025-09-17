use crossterm::event::{self, Event, KeyCode};
use crossterm::style::Color;
use engine::core::object::ObjectExt;
use engine::core::scene::Scene;
use engine::core::{Logic, RuntimeCommand};
use engine::prelude::*;

mod events;
mod food;
mod game_object;
mod player;
mod snake;
mod ui;

use food::Food;
use player::Player;
use rand::Rng;
use snake::{Direction, Snake};
use std::time::{Duration, Instant};

use crate::StageKey;
use events::{CollisionHandler, DeathHandler, FoodEatenHandler};
use ui::{logger::Logger, statistics::Statistics};

pub struct SnakeLogic {
    event_manager: EventManager,
    player: Player,
    speed: u64,
    counter: u64,
    skip: bool,
    quit: bool,
    stats_id: Option<Id>,
    logger_id: Option<Id>,
    max_logs: usize,
    last_tick: Instant,
    is_debugging: bool,
}

impl SnakeLogic {
    pub fn new() -> Self {
        let mut event_manager = EventManager::new();

        event_manager.register(CollisionHandler);
        event_manager.register(FoodEatenHandler);
        event_manager.register(DeathHandler);

        Self {
            event_manager,
            player: Player::new(),
            speed: 20,
            counter: 0,
            skip: true,
            quit: false,
            stats_id: None,
            logger_id: None,
            max_logs: 10,
            last_tick: Instant::now(),
            is_debugging: true,
        }
    }
}

impl Logic<StageKey> for SnakeLogic {
    fn setup(&mut self, scene: &mut Scene) {
        // Attaching spatial grid
        let grid = SpatialGrid::new(200, 80, 1, |_, is_border| {
            if is_border {
                let style = Glyph::new(Some(Color::Grey), Some(Color::Black), '█');
                Terrain::new(style, 255)
            } else {
                let style = Glyph::new(Some(Color::Black), Some(Color::Black), ' ');
                Terrain::new(style, 0)
            }
        });
        scene.attach_grid(grid);

        // UI Stats
        let stats_ui_id = scene.attach_object(
            |id| Box::new(Statistics::new(id, Position::new(203, 1))),
            Conflict::Ignore,
        );
        self.stats_id = stats_ui_id;

        // UI Event logger
        let logger_ui_id = scene.attach_object(
            |id| Box::new(Logger::new(id, Position::new(203, 6), self.max_logs)),
            Conflict::Ignore,
        );
        self.logger_id = logger_ui_id;

        // Player snake
        let snake_id = scene.attach_object(
            |id| {
                Box::new({
                    let mut snake = Snake::new(Position::new(50, 10), id, 3);
                    snake.head_style = Glyph::new(Some(Color::DarkYellow), Some(Color::Black), '█');
                    snake.body_style = Glyph::new(None, Some(Color::DarkMagenta), ' ');
                    snake.base_index = snake.base_index + 2;
                    snake.ignore_death = true;
                    snake.ignore_body = false;
                    snake
                })
            },
            Conflict::Overwrite,
        );

        // Ties snake to player
        if let Some(id) = snake_id {
            self.player.set_snake(id);
            scene.protected_ids.insert(id);
        }
    }

    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<StageKey> {
        if self.quit {
            return RuntimeCommand::Kill;
        }

        if let Some(ui_id) = self.stats_id {
            let now = Instant::now();
            let tick_duration = now.duration_since(self.last_tick);
            self.last_tick = now;
            let objects_count = scene.objects.len();
            let stateful_count = match scene.indexes.get(&ObjectIndex::Stateful) {
                Some(hash_set) => hash_set.len(),
                None => 0,
            };
            if let Some(ui_object) = scene.objects.get_mut(&ui_id) {
                if let Some(stats_ui) = ui_object.get_mut::<Statistics>() {
                    let lines = vec![
                        format!("Object Count: {}", objects_count),
                        format!("Stateful Objects: {}", stateful_count),
                        format!("Tick Duration: {:.2?}", tick_duration),
                    ];
                    stats_ui.set_text(lines);
                }
            }
        }

        self.counter += 1;

        if let Some(snake_id) = self.player.snake {
            if let None = scene.objects.get_mut(&snake_id) {
                return RuntimeCommand::Kill;
            }
        }

        let mut rng = rand::rng();
        if !self.skip {
            let player_snake_id = self.player.snake;

            for (id, object) in scene.objects.iter_mut() {
                if Some(*id) == player_snake_id {
                    continue;
                }

                if let Some(snake) = object.get_mut::<Snake>() {
                    let rnd_dir = rng.random_range(0..4);

                    match rnd_dir {
                        0 => snake.direction = Direction::Up,
                        1 => snake.direction = Direction::Left,
                        2 => snake.direction = Direction::Down,
                        _ => snake.direction = Direction::Right,
                    }
                }
            }
        }

        let random_counter = rng.random_range(1..10);

        if self.counter % random_counter == 0 {
            self.skip = false;
        } else {
            self.skip = true;
        }
        RuntimeCommand::SetTickRate(Duration::from_millis(self.speed))
    }

    fn process_input(&mut self, scene: &mut Scene) {
        if event::poll(Duration::from_millis(0)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                if let Some(snake_id) = self.player.snake {
                    if let Some(snake) = scene.objects.get_mut(&snake_id) {
                        if let Some(snake) = snake.get_mut::<Snake>() {
                            match key_event.code {
                                KeyCode::Char('w') => snake.direction = Direction::Up,
                                KeyCode::Char('s') => snake.direction = Direction::Down,
                                KeyCode::Char('a') => snake.direction = Direction::Left,
                                KeyCode::Char('d') => snake.direction = Direction::Right,
                                KeyCode::Char('q') => snake.resize_head_native(
                                    snake.head_size.native_size().saturating_sub(2),
                                ),
                                KeyCode::Char('e') => snake.resize_head_native(
                                    snake.head_size.native_size().saturating_add(2),
                                ),
                                KeyCode::Char('r') => self.skip = false,
                                KeyCode::Char('+') => {
                                    snake.base_index = snake.base_index.saturating_add(2)
                                }
                                KeyCode::Char('-') => {
                                    snake.base_index = snake.base_index.saturating_sub(2)
                                }
                                KeyCode::Char('f') => {
                                    for _ in 0..100 {
                                        let random_pos: Option<Position> = match &scene.spatial_grid
                                        {
                                            Some(grid) => grid.random_empty_pos(),
                                            None => None,
                                        };

                                        if let Some(pos) = random_pos {
                                            scene.attach_object(
                                                |id| Box::new(Food::rng_food(id, pos)),
                                                Conflict::Cancel,
                                            );
                                        }
                                    }
                                }
                                KeyCode::Esc => self.quit = true,
                                KeyCode::Tab => {
                                    for i in 0..100 {
                                        let pos: Option<Position> = match &scene.spatial_grid {
                                            Some(grid) => {
                                                let x = (self.counter + i) % grid.game_width as u64;
                                                let y = (self.counter + i).saturating_mul(i)
                                                    % grid.game_height as u64;

                                                Some(Position::new(x as u16, y as u16))
                                            }
                                            None => None,
                                        };

                                        if let Some(pos) = pos {
                                            let _ = scene.attach_object(
                                                |id| {
                                                    Box::new({
                                                        let mut snake =
                                                            Snake::new(pos, id, (1) as usize);
                                                        snake.ignore_death = true;
                                                        snake.ignore_body = true;
                                                        snake
                                                    })
                                                },
                                                Conflict::Cancel,
                                            );
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    fn process_events(&mut self, scene: &mut Scene) {
        if self.is_debugging {
            if let Some(logger_id) = self.logger_id {
                if let Some(logger_object) = scene.objects.get_mut(&logger_id) {
                    if let Some(logger_ui) = logger_object.get_mut::<Logger>() {
                        let event_count = scene.event_bus.len();
                        let start_index = event_count.saturating_sub(self.max_logs);
                        for event in &scene.event_bus[start_index..] {
                            logger_ui.add_log(event.log_message());
                        }
                    }
                }
            }
        }
        self.event_manager.dispatch(scene);
    }
}
