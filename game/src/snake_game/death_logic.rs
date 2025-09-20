use crossterm::event::{self, Event, KeyCode};
use crossterm::style::Color;
use engine::prelude::*;
use rand::Rng;
use std::time::{Duration, Instant};

use crate::StageKey;
use super::events::{CollisionHandler, DeathHandler, FoodEatenHandler};
use super::game_objects::{
    snake::Direction,
    Snake,
};
use super::player::Player;
use super::ui::{logger::Logger, statistics::Statistics};

const GAME_SPEED: u64 = 20;
const MAX_LOGS: usize = 20;

pub struct DeathLogic {
    event_manager: EventManager,
    player: Player,
    counter: u64,
    quit: bool,
    stats_id: Option<Id>,
    logger_id: Option<Id>,
    last_tick: Instant,
    is_debugging: bool,
    is_paused: bool,
    old_logic: Option<Box<dyn Logic<StageKey>>>,
    revert_logic: bool,
}

impl DeathLogic {
    pub fn build(player: Player, stats_id: Option<Id>, logger_id: Option<Id>) -> Self {
        let mut event_manager = EventManager::new();
        event_manager.register(CollisionHandler);
        event_manager.register(FoodEatenHandler);
        event_manager.register(DeathHandler);

        Self {
            event_manager,
            player,
            counter: 0,
            quit: false,
            stats_id,
            logger_id,
            last_tick: Instant::now(),
            is_debugging: true,
            is_paused: false,
            old_logic: None,
            revert_logic: false,
        }
    }

    fn handle_input(&mut self, scene: &mut Scene) -> Option<RuntimeCommand<StageKey>> {
        if !event::poll(Duration::from_millis(0)).unwrap() {
            return None;
        }

        let event = event::read().unwrap();
        if let Event::Key(key_event) = event {
            return self.handle_key_event(key_event, scene);
        }

        None
    }

    fn handle_key_event(
        &mut self,
        key_event: event::KeyEvent,
        scene: &mut Scene,
    ) -> Option<RuntimeCommand<StageKey>> {
        if let Some(snake_id) = self.player.snake {
            if let Some(object) = scene.objects.get_mut(&snake_id) {
                if let Some(snake) = object.get_mut::<Snake>() {
                    match key_event.code {
                        KeyCode::Char('w') => snake.direction = Direction::Up,
                        KeyCode::Char('s') => snake.direction = Direction::Down,
                        KeyCode::Char('a') => snake.direction = Direction::Left,
                        KeyCode::Char('d') => snake.direction = Direction::Right,
                        KeyCode::Char('q') => snake
                            .resize_head_native(snake.head_size.native_size().saturating_sub(2)),
                        KeyCode::Char('e') => snake
                            .resize_head_native(snake.head_size.native_size().saturating_add(2)),
                        KeyCode::Char(' ') => snake.is_moving ^= true,
                        KeyCode::Up => snake.base_index = snake.base_index.saturating_add(2),
                        KeyCode::Down => snake.base_index = snake.base_index.saturating_sub(2),
                        KeyCode::Char('g') => self.revert_logic = true,
                        KeyCode::Tab => self.spawn_snakes(scene, 50),
                        KeyCode::Esc => self.quit = true,
                        KeyCode::Char('p') => self.is_paused ^= true,
                        _ => {}
                    }
                }
            }
        }
        None
    }

    fn spawn_snakes(&self, scene: &mut Scene, count: usize) {
        for i in 0..count {
            if let Some(grid) = &scene.spatial_grid {
                let i_u16 = i as u16;
                let x = (self.counter as u16 + i_u16) % grid.game_width;
                let y = ((self.counter as u16 + i_u16) * i_u16) % grid.game_height;
                let pos = Position::new(x, y);

                scene.attach_object(
                    |id| {
                        let mut snake = Snake::new(pos, id, 3);
                        snake.ignore_death = false;
                        snake.ignore_body = true;

                        let color_picker = (self.counter % 255) as u8;
                        let index = (self.counter % 15) as u8;

                        snake.body_style = Glyph::new(
                            Some(Color::Rgb {
                                r: 0,
                                g: 0,
                                b: color_picker.saturating_add(50),
                            }),
                            Some(Color::Black),
                            '█',
                        );
                        snake.head_style = Glyph::new(
                            Some(Color::Rgb {
                                r: 0,
                                g: 0,
                                b: color_picker.saturating_add(10),
                            }),
                            Some(Color::Black),
                            '█',
                        );
                        snake.base_index = index;
                        Box::new(snake)
                    },
                    Conflict::Cancel,
                );
            }
        }
    }

    fn update_ai_snakes(&self, scene: &mut Scene) {
        let player_snake_id = self.player.snake;
        let mut rng = rand::rng();

        // Snakes is the only movable object here
        // ideally we would use our own indexes
        let movables = scene
            .indexes
            .get(&ObjectIndex::Movable)
            .into_iter()
            .flat_map(|hash_set| hash_set.iter());

        for id in movables {
            if Some(*id) == player_snake_id {
                continue;
            }

            if let Some(object) = scene.objects.get_mut(id) {
                if let Some(snake) = object.get_mut::<Snake>() {
                    snake.ignore_death = false;
                    if rng.random_bool(0.4) {
                        snake.direction = match rng.random_range(0..4) {
                            0 => Direction::Up,
                            1 => Direction::Left,
                            2 => Direction::Down,
                            _ => Direction::Right,
                        };
                    }
                }
            }
        }
    }

    fn update_statistics(&mut self, scene: &mut Scene) {
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
                        format!("Logic has switched to death!"),
                        format!("Object Count: {}", objects_count),
                        format!("Stateful Objects: {}", stateful_count),
                        format!("Tick Duration: {:.2?}", tick_duration),
                    ];
                    stats_ui.set_text(lines);
                }
            }
        }
    }
}

impl Logic<StageKey> for DeathLogic {
    fn collect_old_stage(
            &mut self,
            _old_scene: Option<Box<Scene>>,
            _old_logic: Option<Box<dyn Logic<StageKey>>>,
        ) {
        if let Some(old_logic) = _old_logic {
            self.old_logic = Some(old_logic);
        }
    }

    fn setup(&mut self, _scene: &mut Scene) {
        // Revert to old logic as no setup is defined here
        // (We are relying on the old logic)
        self.revert_logic = true
    }

    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<StageKey> {
        if self.revert_logic {
            if let Some(logic) = self.old_logic.take() {
                return RuntimeCommand::ReplaceLogic(logic);
            } else {
                panic!("No revert logic was collected.")
            }
        }

        if let Some(command) = self.handle_input(scene) {
            return command;
        }

        if self.quit {
            return RuntimeCommand::Kill;
        }

        if self.is_paused {
            return RuntimeCommand::Skip;
        }

        self.update_statistics(scene);
        self.counter += 1;

        if self.player.snake.is_some() && scene.objects.get(&self.player.snake.unwrap()).is_none() {
            return RuntimeCommand::Kill;
        }

        self.update_ai_snakes(scene);

        RuntimeCommand::SetTickRate(Duration::from_millis(GAME_SPEED))
    }

    fn dispatch_events(&mut self, scene: &mut Scene) {
        if self.is_debugging {
            if let Some(logger_id) = self.logger_id {
                if let Some(logger_object) = scene.objects.get_mut(&logger_id) {
                    if let Some(logger_ui) = logger_object.get_mut::<Logger>() {
                        let event_count = scene.event_bus.len();
                        let start_index = event_count.saturating_sub(MAX_LOGS);
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
