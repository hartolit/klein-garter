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
        }
    }
}

impl Logic<String> for GameLogic {
    fn setup(&mut self, scene: &mut Scene) {
        let snake_id = scene.attach_object(|id| {
            Box::new(Snake::new(Position::new(30, 10), id, 5))
        });

        self.player.set_snake(snake_id);
    }

    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<String> {
        self.counter += 1;
        if self.speed > 40 {
            self.speed -= 1;
        }

        if let Some(snake_id) = self.player.snake {
            if let Some(object) = scene.objects.get_mut(&snake_id) {
                if let Some(snake) = object.get_mut::<Snake>() {
                    if self.counter < 20 {
                        snake.meals = 1;
                    }

                    if self.counter % 200 == 0 {
                        snake.resize_head_brief(3);
                    } else if self.counter % 100 == 0 {
                        snake.resize_head_native();
                    }

                    if !snake.is_alive {
                        return RuntimeCommand::Kill;
                    }
                }
            }
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
    let mut manager: RuntimeManager<String> = RuntimeManager::new(Duration::from_millis(150));

    // TODO - Make spatial grid attachable :)
    let grid = SpatialGrid::new(100, 50, 3, Kind::Ground);
    let game_logic = Box::new(GameLogic::new());
    let game_stage = Stage::new(game_logic, grid);

    manager.add_stage(GAME_STAGE_KEY.to_string(), game_stage);
    manager.set_active_key(GAME_STAGE_KEY.to_string());

    manager.run_app();
}