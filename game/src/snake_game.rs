use crossterm::event::{self, Event, KeyCode};
use crossterm::style::Color;
use engine::prelude::*;
use rand::Rng;
use std::time::{Duration, Instant};

mod death_logic;
mod events;
mod game_objects;
mod player;
mod ui;

use crate::StageKey;
use death_logic::DeathLogic;
use events::{CollisionHandler, DeathHandler, FoodHandler};
use game_objects::{
    snake::Direction,
    {Food, Snake},
};
use player::Player;
use ui::{InfoPanel, Logger, Statistics};

// Game
const GAME_SPEED: u64 = 20;

// Grid
const GRID_POS: Position = Position { x: 4, y: 3 };
const GRID_WIDTH: u16 = 180;
const GRID_HEIGHT: u16 = 60;
const BORDER_STYLE: Glyph = Glyph {
    fg_clr: Some(Color::Rgb {
        r: 200,
        g: 200,
        b: 200,
    }),
    bg_clr: None,
    symbol: '█',
};

// Statistics
const STATS_COLOR: Color = Color::Rgb {
    r: 170,
    g: 170,
    b: 170,
};

// Logger
const LOGGER_COLOR: Color = Color::Rgb {
    r: 150,
    g: 150,
    b: 200,
};
const MAX_LOGS: usize = 10;

pub struct SnakeLogic {
    stage_id: StageKey,
    switch_stage: Option<StageKey>,
    switch_logic: bool,
    event_manager: EventManager,
    player: Player,
    speed: u64,
    counter: u64,
    quit: bool,
    stats_id: Option<Id>,
    logger_id: Option<Id>,
    info_id: Option<Id>,
    last_tick: Instant,
    is_debugging: bool,
    is_paused: bool,
    grid_pos: Position,
    grid_width: u16,
    grid_height: u16,
}

impl SnakeLogic {
    pub fn new(key: StageKey) -> Self {
        let mut event_manager = EventManager::new();
        event_manager.register(CollisionHandler);
        event_manager.register(FoodHandler);
        event_manager.register(DeathHandler);

        Self {
            stage_id: key,
            switch_stage: None,
            switch_logic: false,
            event_manager,
            player: Player::new(),
            speed: GAME_SPEED,
            counter: 0,
            quit: false,
            stats_id: None,
            logger_id: None,
            info_id: None,
            last_tick: Instant::now(),
            is_debugging: true,
            is_paused: false,
            grid_pos: GRID_POS,
            grid_height: GRID_HEIGHT,
            grid_width: GRID_WIDTH,
        }
    }

    fn setup_scene(&mut self, scene: &mut Scene) {
        self.setup_grid(scene);
        self.setup_ui(scene);
        self.setup_player_snake(scene);
    }

    fn setup_grid(&self, scene: &mut Scene) {
        let grid = SpatialGrid::new(
            self.grid_width,
            self.grid_height,
            Some(BORDER_STYLE),
            self.grid_pos,
            |_| {
                let style = Glyph::new(Some(Color::Black), Some(Color::Black), ' ');
                Terrain::new(style, 0)
            },
        );
        scene.attach_grid(grid);
    }

    fn setup_ui(&mut self, scene: &mut Scene) {
        self.stats_id = scene.attach_object(
            |id| Box::new(Statistics::new(id, Position::empty())),
            Conflict::Ignore,
        );

        self.logger_id = scene.attach_object(
            |id| Box::new(Logger::new(id, Position::empty(), MAX_LOGS)),
            Conflict::Ignore,
        );

        self.info_id = scene.attach_object(
            |id| Box::new(InfoPanel::new(id, Position::empty())),
            Conflict::Ignore,
        );

        self.update_ui_pos(scene);
        self.update_info(scene);
    }

    fn setup_player_snake(&mut self, scene: &mut Scene) {
        let snake_id = scene.attach_object(
            |id| {
                let snake_pos = Position {
                    x: self.grid_pos.x.saturating_add(50),
                    y: self.grid_pos.y.saturating_add(10)
                };

                let mut snake = Snake::new(snake_pos, id, 3);
                snake.head_style = Glyph::new(
                    Some(Color::Rgb {
                        r: 255,
                        g: 0,
                        b: 255,
                    }),
                    Some(Color::Black),
                    '█',
                );
                snake.body_style = Glyph::new(
                    Some(Color::Rgb {
                        r: 138,
                        g: 43,
                        b: 226,
                    }),
                    Some(Color::Black),
                    '█',
                );
                snake.base_index = 20;
                snake.ignore_death = true;
                Box::new(snake)
            },
            Conflict::Overwrite,
        );

        if let Some(id) = snake_id {
            self.player.set_snake(id);
            scene.protected_ids.insert(id);
        }
    }

    fn update_ui_pos(&mut self, scene: &mut Scene) {
        if let Some(stats_id) = self.stats_id {
            if let Some(object) = scene.objects.get_mut(&stats_id) {
                if let Some(stats) = object.get_mut::<Statistics>() {
                    let stats_pos = Position {
                        x: (self.grid_width + self.grid_pos.x) + 3,
                        y: self.grid_pos.y + 17,
                    };
                    stats.pos = stats_pos;
                }
            }
        }

        if let Some(logger_id) = self.logger_id {
            if let Some(object) = scene.objects.get_mut(&logger_id) {
                if let Some(logger) = object.get_mut::<Logger>() {
                    let logger_pos = Position {
                        x: (self.grid_width + self.grid_pos.x) + 3,
                        y: self.grid_pos.y + 22,
                    };
                    logger.pos = logger_pos;
                }
            }
        }

        if let Some(info_id) = self.info_id {
            if let Some(object) = scene.objects.get_mut(&info_id) {
                if let Some(info) = object.get_mut::<InfoPanel>() {
                    let info_pos = Position {
                        x: (self.grid_width + self.grid_pos.x) + 3,
                        y: self.grid_pos.y,
                    };
                    info.start_pos = info_pos;
                }
            }
        }
    }

    fn update_info(&mut self, scene: &mut Scene) {
        if let Some(id) = self.info_id {
            if let Some(ui_object) = scene.objects.get_mut(&id) {
                if let Some(panel) = ui_object.get_mut::<InfoPanel>() {
                    panel.clear();
                    let key_clr = Some(Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    });
                    let title_clr = Some(Color::Rgb {
                        r: 175,
                        g: 200,
                        b: 200,
                    });

                    panel.add_line(format!(":::[CONTROLS]:::"), title_clr, None);
                    panel.add_line(format!("w,a,s,d:        Move Snake"), key_clr, None);
                    panel.add_line(format!("q & e:          Resize Head"), key_clr, None);
                    panel.add_line(format!("Space:          Toggle Move"), key_clr, None);
                    panel.add_line(format!("p:              Pause Game"), key_clr, None);
                    panel.add_line(format!("Esc:            Quit Game"), key_clr, None);
                    panel.add_line(format!(""), None, None); // Spacer
                    panel.add_line(format!(":::[DEBUG]:::"), title_clr, None);
                    panel.add_line(format!("W,A,S,D:        Move camera"), key_clr, None);
                    panel.add_line(format!("Q & E:          Resize camera"), key_clr, None);
                    panel.add_line(format!("Up & Down:      Change Z-Index"), key_clr, None);
                    panel.add_line(format!("Left & Right:   Switch Stage"), key_clr, None);
                    panel.add_line(format!("g:              Switch Logic"), key_clr, None);
                    panel.add_line(format!("r:              Reset Stage"), key_clr, None);
                    panel.add_line(format!("f:              Spawn Food"), key_clr, None);
                    panel.add_line(format!("Tab:            Spawn Snakes"), key_clr, None);
                }
            }
        }
    }

    fn update_statistics(&mut self, scene: &mut Scene) {
        if let Some(id) = self.stats_id {
            let now = Instant::now();
            let tick_duration = now.duration_since(self.last_tick);
            self.last_tick = now;
            let objects_count = scene.objects.len();
            let stateful_count = match scene.indexes.get(&ObjectIndex::Stateful) {
                Some(hash_set) => hash_set.len(),
                None => 0,
            };
            if let Some(ui_object) = scene.objects.get_mut(&id) {
                if let Some(stats_ui) = ui_object.get_mut::<Statistics>() {
                    let lines = vec![
                        format!("Current stage: {}", self.stage_id),
                        format!("Tick Duration: {:.2?}", tick_duration),
                        format!("Object Count: {}", objects_count),
                        format!("Stateful Objects: {}", stateful_count),
                    ];
                    stats_ui.set_text(lines, Some(STATS_COLOR));
                }
            }
        }
    }

    fn handle_input(&mut self, scene: &mut Scene) -> Option<RuntimeCommand<StageKey>> {
        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            let event = match event::read() {
                Ok(event) => event,
                Err(_) => continue,
            };

            match event {
                Event::Key(key_event) => {
                    if let Some(command) = self.handle_key_event(key_event, scene) {
                        return Some(command);
                    }
                }
                Event::Resize(_, _) => {
                    return Some(RuntimeCommand::Refresh);
                }
                _ => {}
            }
        }
        None
    }

    fn handle_key_event(
        &mut self,
        key_event: event::KeyEvent,
        scene: &mut Scene,
    ) -> Option<RuntimeCommand<StageKey>> {
        if !key_event.is_press() {
            return None;
        }

        if let Some(snake_id) = self.player.snake {
            if let Some(object) = scene.objects.get_mut(&snake_id) {
                if let Some(snake) = object.get_mut::<Snake>() {
                    match key_event.code {
                        KeyCode::Char('w') => snake.direction = Direction::Up,
                        KeyCode::Char('s') => snake.direction = Direction::Down,
                        KeyCode::Char('a') => snake.direction = Direction::Left,
                        KeyCode::Char('d') => snake.direction = Direction::Right,
                        KeyCode::Char('W') => return Some(self.handle_grid_move(Direction::Up, scene)),
                        KeyCode::Char('S') => return Some(self.handle_grid_move(Direction::Down, scene)),
                        KeyCode::Char('A') => return Some(self.handle_grid_move(Direction::Left, scene)),
                        KeyCode::Char('D') => return Some(self.handle_grid_move(Direction::Right, scene)),
                        KeyCode::Char('q') => snake
                            .resize_head_native(snake.head_size.native_size().saturating_sub(2)),
                        KeyCode::Char('e') => snake
                            .resize_head_native(snake.head_size.native_size().saturating_add(2)),
                        KeyCode::Char('Q') => return Some(self.handle_new_grid(scene, false)),
                        KeyCode::Char('E') => return Some(self.handle_new_grid(scene, true)),
                        KeyCode::Char(' ') => snake.is_moving ^= true,
                        KeyCode::Up => snake.base_index = snake.base_index.saturating_add(2),
                        KeyCode::Down => snake.base_index = snake.base_index.saturating_sub(2),
                        KeyCode::Left => self.handle_stage_switch(),
                        KeyCode::Right => self.handle_stage_switch(),
                        KeyCode::Char('f') => self.spawn_food(scene, 100),
                        KeyCode::Tab => self.spawn_snakes(scene, 200),
                        KeyCode::Char('r') => return Some(RuntimeCommand::Reset),
                        KeyCode::Esc => self.quit = true,
                        KeyCode::Char('p') => self.is_paused ^= true,
                        KeyCode::Char('g') => self.switch_logic = true,
                        _ => {}
                    }
                }
            }
        }
        None
    }

    fn handle_new_grid(&mut self, scene: &mut Scene, is_grow: bool) -> RuntimeCommand<StageKey> {
        if is_grow {
            self.grid_height = self.grid_height.saturating_add(2);
            self.grid_width = self.grid_width.saturating_add(2);
        } else {
            self.grid_height = self.grid_height.saturating_sub(2);
            self.grid_width = self.grid_width.saturating_sub(2);
        }

        self.setup_grid(scene);
        return RuntimeCommand::Refresh;
    }

    fn handle_grid_move(&mut self, direction: Direction, scene: &mut Scene) -> RuntimeCommand<StageKey> {
        if let Some(grid) = &mut scene.spatial_grid {
            let (dx, dy) = direction.get_move(5);

            self.grid_pos.x = self.grid_pos.x.saturating_add_signed(dx);
            self.grid_pos.y = self.grid_pos.y.saturating_add_signed(dy);

            grid.origin = self.grid_pos;
            return RuntimeCommand::Refresh;
        }

        return RuntimeCommand::None;
    }

    fn handle_stage_switch(&mut self) {
        self.switch_stage = match self.stage_id {
            StageKey::Level0 => Some(StageKey::Level1),
            StageKey::Level1 => Some(StageKey::Level0),
        }
    }

    fn spawn_food(&self, scene: &mut Scene, count: usize) {
        for _ in 0..count {
            if let Some(grid) = &scene.spatial_grid {
                if let Some(pos) = grid.random_empty_pos() {
                    scene.attach_object(|id| Box::new(Food::rng_food(id, pos)), Conflict::Cancel);
                }
            }
        }
    }

    fn spawn_snakes(&self, scene: &mut Scene, count: usize) {
        let gx = self.grid_pos.x;
        let gy = self.grid_pos.y;

        for i in 0..count {
            if let Some(grid) = &scene.spatial_grid {
                let i_u16 = i as u16;
                let x = (self.counter as u16 + i_u16) % grid.width;
                let y = ((self.counter as u16 + i_u16) * i_u16) % grid.height;
                let pos = Position::new(x.saturating_add(gx), y.saturating_add(gy));

                scene.attach_object(
                    |id| {
                        let mut snake = Snake::new(pos, id, 1);
                        snake.ignore_death = true;
                        snake.ignore_body = true;

                        let color_picker = (self.counter % 255) as u8;
                        let index = (self.counter % 15) as u8;

                        snake.body_style = Glyph::new(
                            Some(Color::Rgb {
                                r: color_picker.saturating_sub(50),
                                g: 20,
                                b: 30,
                            }),
                            Some(Color::Black),
                            '█',
                        );
                        snake.head_style = Glyph::new(
                            Some(Color::Rgb {
                                r: color_picker,
                                g: 20,
                                b: 30,
                            }),
                            Some(Color::Black),
                            '█',
                        );
                        snake.base_index = index;
                        Box::new(snake)
                    },
                    Conflict::Overwrite,
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
                    if rng.random_bool(0.1) {
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
}

impl Logic<StageKey> for SnakeLogic {
    fn init(&mut self, scene: &mut Scene) {
        self.setup_scene(scene);
    }

    fn refresh(&mut self, scene: &mut Scene) {
        if let Some(stats_id) = self.stats_id {
            if let Some(object) = scene.objects.get_mut(&stats_id) {
                if let Some(stats) = object.get_mut::<Statistics>() {
                    stats.clear();
                }
            }
        }

        if let Some(logger_id) = self.logger_id {
            if let Some(object) = scene.objects.get_mut(&logger_id) {
                if let Some(logger) = object.get_mut::<Logger>() {
                    logger.clear();
                }
            }
        }

        self.update_ui_pos(scene);
        self.update_info(scene);
    }

    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<StageKey> {
        if let Some(command) = self.handle_input(scene) {
            return command;
        }

        if self.quit {
            return RuntimeCommand::Kill;
        }

        if let Some(key) = self.switch_stage {
            self.switch_stage = None;
            return RuntimeCommand::SwitchStage(key);
        }

        if self.switch_logic {
            self.switch_logic = false;
            let new_logic = DeathLogic::build(
                self.stage_id,
                self.player,
                self.stats_id,
                self.logger_id,
                self.info_id,
            );
            return RuntimeCommand::ReplaceLogic(Box::new(new_logic));
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

        RuntimeCommand::SetTickRate(Duration::from_millis(self.speed))
    }

    fn dispatch_events(&mut self, scene: &mut Scene) {
        if self.is_debugging {
            if let Some(logger_id) = self.logger_id {
                if let Some(logger_object) = scene.objects.get_mut(&logger_id) {
                    if let Some(logger_ui) = logger_object.get_mut::<Logger>() {
                        let event_count = scene.event_bus.len();
                        let start_index = event_count.saturating_sub(MAX_LOGS);
                        for event in &scene.event_bus[start_index..] {
                            logger_ui.add_log(event.log_message(), Some(LOGGER_COLOR));
                        }
                    }
                }
            }
        }
        self.event_manager.dispatch(scene);
    }
}
