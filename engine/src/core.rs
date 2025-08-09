use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

pub mod global;
pub mod grid;
pub mod object;
pub mod renderer;
pub mod scene;

use object::Action;
use renderer::Renderer;
use scene::{ObjectIndex, Scene};

use crate::core::grid::SpatialGrid;

pub struct Stage<K: Eq + Hash + Clone> {
    logic: Box<dyn Logic<K>>,
    scene: Box<Scene>,
}

impl<K: Eq + Hash + Clone> Stage<K> {
    pub fn new(logic: Box<dyn Logic<K>>, grid: SpatialGrid) -> Self {
        Self {
            logic,
            scene: Box::new(Scene::new(grid)),
        }
    }

    pub fn replace_scene(&mut self, scene: Box<Scene>) -> Box<Scene> {
        let old_scene = std::mem::replace(&mut self.scene, scene);
        old_scene
    }

    pub fn replace_logic(&mut self, logic: Box<dyn Logic<K>>) -> Box<dyn Logic<K>> {
        let old_logic = std::mem::replace(&mut self.logic, logic);
        old_logic
    }

    pub fn replace_stage(
        &mut self,
        logic: Box<dyn Logic<K>>,
        scene: Box<Scene>,
    ) -> (Box<dyn Logic<K>>, Box<Scene>) {
        let old_logic = std::mem::replace(&mut self.logic, logic);
        let old_scene = std::mem::replace(&mut self.scene, scene);
        (old_logic, old_scene)
    }
}

pub trait Logic<K: Eq + Hash + Clone> {
    fn process_actions(&self, scene: &mut Scene, actions: Vec<Action>);
    fn process_input(&self);
    fn setup(&self, scene: &mut Scene);
    fn update(&self, scene: &mut Scene) -> RuntimeCommand<K>;
    fn collect_old_stage(
        &mut self,
        _old_scene: Option<Box<Scene>>,
        _old_logic: Option<Box<dyn Logic<K>>>,
    ) {
    }
}

pub enum RuntimeCommand<K: Eq + Hash + Clone> {
    ReplaceScene(Box<Scene>),
    ReplaceLogic(Box<dyn Logic<K>>),
    ReplaceStage {
        scene: Box<Scene>,
        logic: Box<dyn Logic<K>>,
    },
    SwitchStage(K),
    Kill,
    None,
}

enum ManagerDirective<K: Eq + Hash + Clone> {
    Switch(K),
    Kill,
}

pub struct RuntimeManager<K: Eq + Hash + Clone> {
    runtime: Runtime,
    stages: HashMap<K, Stage<K>>,
    active_key: Option<K>,
}

impl<K: Eq + Hash + Clone> RuntimeManager<K> {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            runtime: Runtime::new(tick_rate),
            stages: HashMap::new(),
            active_key: None,
        }
    }

    pub fn add_stage(&mut self, key: K, stage: Stage<K>) {
        self.stages.insert(key, stage);
    }

    pub fn run_app(&mut self) {
        loop {
            let mut active_stage = match self.active_key.as_mut() {
                Some(key) => self
                    .stages
                    .remove(&key)
                    .expect("Fatal: Active stage does not exist!"),
                None => continue,
            };

            let directive = self.runtime.run(&mut active_stage);

            if let Some(key) = self.active_key.as_mut() {
                self.stages.insert(key.clone(), active_stage);
            }

            match directive {
                ManagerDirective::Switch(new_key) => {
                    if !self.stages.contains_key(&new_key) {
                        panic!("Fatal: Attempted to switch to a non-existent stage key!")
                    }
                    self.active_key = Some(new_key);
                }
                ManagerDirective::Kill => {
                    self.runtime.renderer.kill();
                    break;
                }
            }
        }
    }
}

pub struct Runtime {
    pub tick_rate: Duration,
    last_update: Instant,
    renderer: Renderer,
}

impl Runtime {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            tick_rate,
            last_update: Instant::now(),
            renderer: Renderer::new(),
        }
    }

    fn run<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) -> ManagerDirective<K> {
        self.initialize(stage);
        self.last_update = Instant::now();

        loop {
            stage.logic.process_input();

            let now = Instant::now();
            let delta = now.duration_since(self.last_update);

            if delta >= self.tick_rate {
                self.last_update = now;
                let command = stage.logic.update(&mut stage.scene);

                if let Some(directive) = self.execute_command(command, stage) {
                    return directive;
                }

                self.tick(stage);
                stage.scene.sync();
                self.renderer.partial_render(
                    &stage.scene.spatial_grid,
                    &stage.scene.global_state.finalized,
                );
            }

            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn initialize<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) {
        stage.logic.setup(&mut stage.scene);
        self.renderer.init();
        self.renderer
            .full_render(&mut stage.scene.spatial_grid, &stage.scene.objects);
    }

    fn tick<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) {
        let future_moves = stage
            .scene
            .indexes
            .get(&ObjectIndex::Movable)
            .into_iter()
            .flat_map(|set| set.iter())
            .filter_map(|id| {
                stage
                    .scene
                    .objects
                    .get(id)
                    .and_then(|obj| obj.as_movable())
                    .map(|movable| (*id, movable))
            })
            .flat_map(|(id, movable)| movable.predict_pos().map(move |pos| (id, pos)));

        let mut probe_map = stage.scene.spatial_grid.probe_moves(future_moves);

        let mut actions: Vec<Action> = Vec::new();
        for (id, probe) in probe_map.drain() {
            if let Some(object) = stage.scene.objects.get_mut(&id) {
                if let Some(movable) = object.as_movable_mut() {
                    actions.extend(movable.make_move(probe));
                }
            }
        }

        stage.logic.process_actions(&mut stage.scene, actions);
    }

    fn execute_command<K: Eq + Hash + Clone>(
        &mut self,
        command: RuntimeCommand<K>,
        stage: &mut Stage<K>,
    ) -> Option<ManagerDirective<K>> {
        match command {
            RuntimeCommand::ReplaceScene(scene) => {
                let old_scene = stage.replace_scene(scene);
                stage.logic.collect_old_stage(Some(old_scene), None);
            }
            RuntimeCommand::ReplaceLogic(logic) => {
                let old_logic = stage.replace_logic(logic);
                stage.logic.collect_old_stage(None, Some(old_logic));
            }
            RuntimeCommand::ReplaceStage { scene, logic } => {
                let old_stage = stage.replace_stage(logic, scene);
                stage
                    .logic
                    .collect_old_stage(Some(old_stage.1), Some(old_stage.0));
            }
            RuntimeCommand::SwitchStage(key) => return Some(ManagerDirective::Switch(key)),
            RuntimeCommand::Kill => return Some(ManagerDirective::Kill),
            RuntimeCommand::None => {}
        }
        None
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum State {
//     Init,
//     Run,
//     Stop,
//     Pause,
// }

// pub struct GameLogic {
//     state: State,
//     out: Stdout,
//     tick_rate: Duration,
//     last_update: Instant,
//     world: World,
// }

// pub struct Game<'a> {
//     pub players: Vec<Player>,
//     objects: HashMap<Id, Box<dyn Object>>,
//     food_spawns: usize,
//     state: State,
//     kind: GameKind,
//     out: &'a mut Stdout,
//     tick_rate: Duration,
//     last_update: Instant,
//     id_counter: IdCounter,
//     spatial_grid: SpatialGrid,
// }

// // TODO - Add threads for (players, gameloop)
// impl<'a> Game<'a> {
//     pub fn new(kind: GameKind, stdout: &'a mut Stdout) -> Self {
//         Game {
//             players: Vec::new(),
//             objects: HashMap::new(),
//             food_spawns: 5,
//             state: State::Init,
//             kind: kind,
//             out: stdout,
//             tick_rate: Duration::new(0, 500),
//             last_update: Instant::now(),
//             id_counter: IdCounter::new(),
//             spatial_grid: SpatialGrid::new(40, 20, 2, CellKind::Ground),
//         }
//     }

//     pub fn start(&mut self) -> io::Result<()> {
//         crossterm::terminal::enable_raw_mode()?;
//         queue!(self.out, cursor::Hide)?;

//         loop {
//             let now = Instant::now();
//             let delta = now.duration_since(self.last_update);

//             // Player input
//             if event::poll(self.tick_rate.saturating_sub(delta))? {
//                 if let event::Event::Key(key_event) = event::read()? {
//                     match key_event.code {
//                         event::KeyCode::Char('q') | event::KeyCode::Esc => {
//                             self.state = State::Stop;
//                         }
//                         event::KeyCode::Char('r') => {
//                             // TODO - Add restart state?
//                             self.state = State::Init;
//                         }
//                         event::KeyCode::Char('p') => {
//                             if let State::Pause = self.state {
//                                 self.state = State::Pause
//                             } else {
//                                 self.state = State::Run
//                             }
//                         }
//                         _ => {
//                             for player in self.players.iter_mut() {
//                                 for (direction, key) in player.keys.iter() {
//                                     if key_event.code == KeyCode::Char(*key) {
//                                         if let Some(snake_id) = player.snake {
//                                             if let Some(object) = self.objects.get_mut(&snake_id) {
//                                                 if let Some(snake) = object.get_mut::<Snake>() {
//                                                     snake.direction = *direction;
//                                                 }
//                                             }
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }

//             if delta >= self.tick_rate {
//                 self.last_update = now;

//                 match self.state {
//                     State::Init => self.init().unwrap(),
//                     State::Run => self.run().unwrap(),
//                     State::Pause => self.pause().unwrap(),
//                     State::Stop => break,
//                 }
//             }
//         }

//         crossterm::terminal::disable_raw_mode()?;
//         execute!(self.out, cursor::Show)?;

//         Ok(())
//     }

//     fn init(&mut self) -> io::Result<()> {
//         queue!(self.out, terminal::Clear(terminal::ClearType::All)).unwrap();
//         self.generate_players();
//         self.generate_food();

//         self.state = State::Run;
//         Ok(())
//     }

//     fn pause(&mut self) -> io::Result<()> {
//         Ok(())
//     }

//     fn run(&mut self) -> io::Result<()> {
//         Ok(())
//     }

//     fn generate_food(&mut self) {
//         let missing_food = self.food_spawns - self.object_count(ObjectKind::Food);

//         for _ in 0..missing_food {
//             let pos = self.spatial_grid.rng_empty_pos(None);
//             let food = Food::rng_food(self.id_counter.next(), pos);
//             self.objects.insert(food.id(), Box::new(food));
//         }
//     }

//     fn generate_players(&mut self) {
//         let num_players = self.players.len();
//         if num_players == 0 {
//             return;
//         }

//         let border = self.spatial_grid.border;
//         let playable_width = self.spatial_grid.full_width.saturating_sub(border * 2);
//         let playable_height = self.spatial_grid.full_height.saturating_sub(border * 2);

//         let step_x = playable_width as f32 / (num_players as f32 + 1.0);
//         let step_y = playable_height as f32 / (num_players as f32 + 1.0);

//         for (index, player) in self.players.iter_mut().enumerate() {
//             let relative_x = ((index + 1) as f32 * step_x).round() as u16;
//             let relative_y = ((index + 1) as f32 * step_y).round() as u16;

//             let final_pos = Position {
//                 x: border + relative_x,
//                 y: border + relative_y,
//             };

//             let clamped_pos = Position {
//                 x: final_pos
//                     .x
//                     .clamp(border, self.spatial_grid.full_width - 1 - border),
//                 y: final_pos
//                     .y
//                     .clamp(border, self.spatial_grid.full_height - 1 - border),
//             };

//             let snake = Snake::new(clamped_pos, self.id_counter.next(), 2);
//             player.snake = Some(snake.id());

//             self.objects.insert(snake.id(), Box::new(snake));
//         }
//     }

//     fn object_count(&self, kind: ObjectKind) -> usize {
//         let mut count: usize = 0;
//         for object in self.objects.values() {
//             if object.kind() == kind {
//                 count += 1;
//             }
//         }
//         count
//     }

//     fn draw(&self) {
//         if State::Init == self.state {}
//     }

//     // TODO - add collision
//     fn collision_check(&mut self) {}

//     // TODO - add drawing
// }

// pub fn start() {
//     print!("\x1B[?25l"); // Removes cursor

//     let mut level = Level::new(40, 20);
//     let mut player = Player::new(level.rng_pos(Some(2)));
//     let mut stdout = stdout();

//     level.generate(&mut stdout).unwrap();

//     let mut buf = [0u8; 1];
//     while player.snake.is_alive {
//         io::stdin().read_exact(&mut buf).expect("Failed to read input");

//         let input = buf[0] as char;

//         match input {
//             'w' => player.snake.direction = Direction::Up,
//             's' => player.snake.direction = Direction::Down,
//             'd' => player.snake.direction = Direction::Right,
//             'a' => player.snake.direction = Direction::Left,
//             _ => {}
//         }

//         player.snake.slither();

//         collision_check(&mut player.snake, &mut level);
//         draw(&mut player.snake, &level, &mut stdout).unwrap();
//         stdout.flush().unwrap();
//     }
// }

// fn collision_check(snake: &mut Snake, level: &mut Level) {
//     if snake.head_pos.x < level.border_width
//         || snake.head_pos.x > level.total_width() - level.border_width - 1
//         || snake.head_pos.y < level.border_height
//         || snake.head_pos.y > level.total_height() - level.border_height - 1 {

//             snake.is_alive = false;
//         return;
//     }

//     for part in &snake.body_vec {
//         if snake.head_pos == *part {
//             snake.is_alive = false;
//         }
//     }

//     level.food_vec.retain(|food|{
//         if snake.head_pos == food.pos {
//             snake.meals += food.meals;
//             false
//         } else {
//             true
//         }
//     });
// }

// fn draw(snake: &mut Snake, level: &Level, stdout: &mut Stdout) -> io::Result<()> {
//     // Draw head
//     if let Some(color_ref) = level.bg_color_range.get(snake.head_pos.y as usize) {
//         stdout.queue(SetBackgroundColor(*color_ref))?;
//     }

//     stdout
//         .queue(SetForegroundColor(snake.head_color))?
//         .queue(MoveTo(snake.head_pos.x, snake.head_pos.y))?
//         .queue(Print(snake.head))?;

//     // Draw first_body_part
//     if let Some(firt_body_part) = &snake.body_vec.front().copied() {
//         if let Some(color_ref) = level.bg_color_range.get(firt_body_part.y as usize){
//             stdout.queue(SetBackgroundColor(*color_ref))?;
//         }

//         stdout
//             .queue(SetForegroundColor(snake.body_color))?
//             .queue(MoveTo(firt_body_part.x, firt_body_part.y))?
//             .queue(Print(&snake.body))?;
//     }

//     if snake.meals > 0 {
//         snake.meals -= 1;
//     } else if snake.meals < 0 {
//         let parts_to_remove = snake.meals.abs() as u16;

//         for _ in 0..parts_to_remove {
//             if snake.body_vec.len() <= 1 {
//                 snake.is_alive = false;
//                 break;
//             }

//             if let Some(last_part) = snake.body_vec.pop_back() {
//                 if let Some(color_ref) = level.bg_color_range.get(last_part.y as usize){
//                     stdout.queue(SetBackgroundColor(*color_ref))?;
//                 }

//                 stdout
//                     .queue(MoveTo(last_part.x, last_part.y))?
//                     .queue(Print(&level.background))?;
//             }
//         }
//         snake.meals = 0;
//     } else {
//         if let Some(last_part) = snake.body_vec.pop_back() {
//             if let Some(color_ref) = level.bg_color_range.get(last_part.y as usize){
//                 stdout.queue(SetBackgroundColor(*color_ref))?;
//             }

//             stdout
//                 .queue(MoveTo(last_part.x, last_part.y))?
//                 .queue(Print(&level.background))?;
//         }
//     }

//     if level.food_vec.len() < level.max_food{
//         let missing_food = level.food_vec.len() - level.max_food;

//         for _ in 0..missing_food {
//         }
//     }

//     snake.body_vec.push_front(snake.head_pos);

//     Ok(())
// }
