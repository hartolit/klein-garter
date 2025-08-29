use crossterm::event::{self, Event, KeyCode};
use engine::core::grid::cell::Kind;
use engine::core::grid::SpatialGrid;
use engine::core::object::ObjectExt;
use engine::core::scene::Scene;
use engine::core::{Logic, RuntimeCommand, RuntimeManager, Stage};
use engine::prelude::*;

mod player;
mod snake;
mod game_object;

use player::{Player, PlayerKind};
use snake::{Direction, Snake};
use std::collections::HashMap;
use std::time::Duration;

const GAME_STAGE_KEY: &str = "game";

struct GameLogic {
    player: Player,
    speed: u64,
    counter: u64,
    skip: bool,
}

impl GameLogic {
    fn new() -> Self {
        let mut keys = HashMap::new();
        keys.insert(Direction::Up, 'w');
        keys.insert(Direction::Down, 's');
        keys.insert(Direction::Left, 'a');
        keys.insert(Direction::Right, 'd');

        Self {
            player: Player::new(PlayerKind::Local, keys),
            speed: 150,
            counter: 0,
            skip: true,
        }
    }
}

impl Logic<String> for GameLogic {
    fn setup(&mut self, scene: &mut Scene) {

        for i in 0..9 {
            let _ = scene.attach_object(|id| {
                Box::new(Snake::new(Position::new(i*10+10, 20), id, (i+2) as usize))
            });
        }

        let snake_id = scene.attach_object(|id| {
            Box::new(Snake::new(Position::new(50, 40), id, 9))
        });

        self.player.set_snake(snake_id);
    }

    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<String> {
        self.counter += 1;
        if self.speed > 20 {
            self.speed -= 1;
        }

        if let Some(snake_id) = self.player.snake {
            if let Some(object) = scene.objects.get_mut(&snake_id) {
                if let Some(snake) = object.get_mut::<Snake>() {
                    if self.counter < 20 {
                        snake.meals += 1;
                    }

                    if self.counter % 400 == 0 {
                        snake.resize_head_brief(3);
                    } else if self.counter % 300 == 0 {
                        snake.resize_head_native();
                    }

                    if !snake.is_alive {
                        return RuntimeCommand::Kill;
                    }
                }
            }
        }

        if !self.skip {
            let test_id = Id::new(2);
            if let Some(object) = scene.objects.get_mut(&test_id) {
                if let Some(snake) = object.get_mut::<Snake>() {
                    match self.counter % 4 {
                        0 => snake.direction = Direction::Up,
                        1 => snake.direction = Direction::Left,
                        2 => snake.direction = Direction::Down,
                        3 => snake.direction = Direction::Right,
                        _ => {},
                    }
                }
            }

            let test_id = Id::new(5);
            if let Some(object) = scene.objects.get_mut(&test_id) {
                if let Some(snake) = object.get_mut::<Snake>() {
                    match self.counter % 4 {
                        0 => snake.direction = Direction::Up,
                        1 => snake.direction = Direction::Left,
                        2 => snake.direction = Direction::Down,
                        3 => snake.direction = Direction::Right,
                        _ => {},
                    }
                }
            }
        }

        if self.counter % 7 == 0 {
            self.skip = false;
        } else {
            self.skip = true;
        }

        RuntimeCommand::SetTickRate(Duration::from_millis(self.speed))
    }

    fn process_actions(&mut self, scene: &mut Scene, actions: Vec<Action>) {
    }

    fn process_input(&mut self, scene: &mut Scene) {
        if event::poll(Duration::from_millis(0)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                if let Some(snake_id) = self.player.snake {
                    if let Some(snake) = scene.objects.get_mut(&snake_id) {
                        if let Some(snake) = snake.get_mut::<Snake>() {
                            let new_direction = match key_event.code {
                                KeyCode::Char('w') => Some(Direction::Up),
                                KeyCode::Char('s') => Some(Direction::Down),
                                KeyCode::Char('a') => Some(Direction::Left),
                                KeyCode::Char('d') => Some(Direction::Right),
                                _ => None,
                            };

                            if let Some(direction) = new_direction {
                                snake.direction = direction;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let mut manager: RuntimeManager<String> = RuntimeManager::new(Duration::from_millis(0));

    // TODO - Make spatial grid attachable :)
    let grid = SpatialGrid::new(100, 40, 1, Kind::Ground);
    let game_logic = Box::new(GameLogic::new());
    let game_stage = Stage::new(game_logic, grid);

    manager.add_stage(GAME_STAGE_KEY.to_string(), game_stage);
    manager.set_active_key(GAME_STAGE_KEY.to_string());

    manager.run_app();
}