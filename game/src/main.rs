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
    quit: bool,
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
            speed: 40,
            counter: 0,
            skip: true,
            quit: false,
        }
    }
}

impl Logic<String> for GameLogic {
    fn setup(&mut self, scene: &mut Scene) {

        let snake_id = scene.attach_object(|id| {
            Box::new(Snake::new(Position::new(50, 40), id, 1))
        });

        self.player.set_snake(snake_id);

        for i in 0..3 {
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

    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<String> {
        if self.quit {
            return RuntimeCommand::Kill;
        }

        self.counter += 1;

        if let Some(snake_id) = self.player.snake {
            if let Some(object) = scene.objects.get_mut(&snake_id) {
                if let Some(snake) = object.get_mut::<Snake>() {
                    if self.counter < 20 {
                        snake.meals += 1;
                    }

                    if !snake.is_alive {
                        return RuntimeCommand::Kill;
                    }
                }
            }
        }

        if !self.skip {
            for i in 1..scene.objects.len() {
                let test_id = Id::new(i as u64);
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
        }

        if self.counter % 5 == 0 {
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
                            match key_event.code {
                                KeyCode::Char('w') => snake.direction = Direction::Up,
                                KeyCode::Char('s') => snake.direction = Direction::Down,
                                KeyCode::Char('a') => snake.direction = Direction::Left,
                                KeyCode::Char('d') => snake.direction = Direction::Right,
                                KeyCode::Char('q') => snake.resize_head(snake.head_size.native().saturating_sub(1)),
                                KeyCode::Char('e') => snake.resize_head(snake.head_size.native().saturating_add(1)),
                                KeyCode::Char('t') => println!("                                                                        Objects: {}", scene.objects.len()),
                                KeyCode::Char('r') => self.skip = false,
                                KeyCode::Char('+') => self.speed = self.speed.saturating_add(2),
                                KeyCode::Char('-') => self.speed = self.speed.saturating_sub(2),
                                KeyCode::Esc => self.quit = true,
                                KeyCode::Tab => {
                                    for i in 0..20 {
                                        let x = (self.counter + i) % scene.spatial_grid.game_width as u64;
                                        let y = (self.counter + i) % scene.spatial_grid.game_height as u64;
                                        let _ = scene.attach_object(|id| {
                                            Box::new(Snake::new(Position::new(x as u16, y as u16), id, (1) as usize))
                                        });
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