use crossterm::event::{self, Event, KeyCode};
use crossterm::style::Color;
use engine::core::object::ObjectExt;
use engine::core::scene::Scene;
use engine::core::{Logic, RuntimeCommand};
use engine::prelude::*;

mod player;
mod snake;
mod food;
mod game_object;
mod events;

use player::{Player, PlayerKind};
use rand::Rng;
use snake::{Direction, Snake};
use food::Food;
use std::collections::HashMap;
use std::time::Duration;

use events::{CollisionHandler, DeathHandler, FoodEatenHandler};
use crate::StageKey;

pub struct GameLogic {
    event_manager: EventManager,
    player: Player,
    speed: u64,
    counter: u64,
    skip: bool,
    quit: bool,
}

impl GameLogic {
    pub fn new() -> Self {
        let mut event_manager = EventManager::new();

        event_manager.register(CollisionHandler);
        event_manager.register(FoodEatenHandler);
        event_manager.register(DeathHandler);

        let mut keys = HashMap::new();
        keys.insert(Direction::Up, 'w');
        keys.insert(Direction::Down, 's');
        keys.insert(Direction::Left, 'a');
        keys.insert(Direction::Right, 'd');

        Self {
            event_manager,
            player: Player::new(PlayerKind::Local, keys),
            speed: 40,
            counter: 0,
            skip: true,
            quit: false,
        }
    }
}

impl Logic<StageKey> for GameLogic {
    fn setup(&mut self, scene: &mut Scene) {
        let grid = SpatialGrid::new(100, 40, 1, Kind::Ground);
        scene.attach_grid(grid);

        let snake_id = scene.attach_object(|id| {
            Box::new({
                let mut snake = Snake::new(Position::new(50, 10), id, 3);
                snake.ignore_death = true;
                snake.body_style = Glyph::new(Some(Color::DarkMagenta), Some(Color::Red), 'W');
                snake
            })
        });

        self.player.set_snake(snake_id);

        for i in 0..0 {
            let _ = scene.attach_object(|id| {
                Box::new(Snake::new(Position::new(i*20+5, 5), id, (1) as usize))
            });

            let _ = scene.attach_object(|id| {
                Box::new(Snake::new(Position::new(i*20+5, 15), id, (1) as usize))
            });

            let _ = scene.attach_object(|id| {
                Box::new(Snake::new(Position::new(i*20+5, 25), id, (1) as usize))
            });
        }
    }

    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<StageKey> {
    if self.quit {
        return RuntimeCommand::Kill;
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
                                KeyCode::Char('q') => snake.resize_head(snake.head_size.native_size().saturating_sub(2)),
                                KeyCode::Char('e') => snake.resize_head(snake.head_size.native_size().saturating_add(2)),
                                KeyCode::Char('t') => println!("                                                                        Objects: {}", scene.objects.len()),
                                KeyCode::Char('r') => self.skip = false,
                                KeyCode::Char('+') => snake.base_index = snake.base_index.saturating_add(2),
                                KeyCode::Char('-') => snake.base_index = snake.base_index.saturating_sub(2),
                                KeyCode::Char('f') => {
                                    for _ in 0..100 {
                                        let random_pos: Option<Position> = match &scene.spatial_grid {
                                            Some(grid) => {
                                                grid.random_empty_pos()
                                            },
                                            None => None,
                                        };

                                        if let Some(pos) = random_pos {
                                            scene.attach_object(|id| {
                                                Box::new(Food::rng_food(id, pos))
                                            });
                                        }
                                    }
                                },
                                KeyCode::Esc => self.quit = true,
                                KeyCode::Tab => {
                                    for i in 0..2 {
                                        let pos: Option<Position> = match &scene.spatial_grid {
                                            Some(grid) => {
                                                let x = (self.counter + i) % grid.game_width as u64;
                                                let y = (self.counter + i) % grid.game_height as u64;

                                                Some(Position::new(x as u16, y as u16))
                                            },
                                            None => None,
                                        };

                                        if let Some(pos) = pos {
                                            let _ = scene.attach_object(|id| {
                                                Box::new(Snake::new(pos, id, (1) as usize))
                                            });
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    fn process_events(&mut self, scene: &mut Scene) {
        self.event_manager.dispatch(scene);
    }
}