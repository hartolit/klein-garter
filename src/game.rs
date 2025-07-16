pub mod food;
pub mod grid;
pub mod object;
pub mod player;

use crossterm::{
    cursor,
    event::{self, KeyCode},
    execute, queue,
    terminal::{self},
};
use std::{
    io::{self},
    time::{Duration, Instant},
};

use grid::{CellKind, SpatialGrid};
use io::Stdout;
use object::{Id, IdCounter, Position};
use player::Player;
use food::Food;

use crate::game::object::Object;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum State {
    Init,
    Run,
    Stop,
    Pause,
}

pub enum GameKind {
    Local,
    Online,
}

pub struct Game<'a> {
    pub players: Vec<Player>,
    foods: Vec<Food>,
    food_spawns: usize,
    state: State,
    kind: GameKind,
    out: &'a mut Stdout,
    tick_rate: Duration,
    last_update: Instant,
    id_counter: IdCounter,
    spatial_grid: SpatialGrid,
}

// TODO - Add threads for (players, gameloop)
impl<'a> Game<'a> {
    pub fn new(kind: GameKind, stdout: &'a mut Stdout) -> Self {
        Game {
            state: State::Init,
            kind: kind,
            players: Vec::new(),
            foods: Vec::new(),
            food_spawns: 5,
            out: stdout,
            tick_rate: Duration::new(0, 500),
            last_update: Instant::now(),
            id_counter: IdCounter::new(),
            spatial_grid: SpatialGrid::new(40, 20, 2, CellKind::Ground),
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        queue!(self.out, cursor::Hide)?;

        loop {
            let now = Instant::now();
            let delta = now.duration_since(self.last_update);

            // Player input
            if event::poll(self.tick_rate.saturating_sub(delta))? {
                if let event::Event::Key(key_event) = event::read()? {
                    match key_event.code {
                        event::KeyCode::Char('q') | event::KeyCode::Esc => {
                            self.state = State::Stop;
                        }
                        event::KeyCode::Char('r') => {
                            // TODO - Add restart state?
                            self.state = State::Init;
                        }
                        event::KeyCode::Char('p') => {
                            if let State::Pause = self.state {
                                self.state = State::Pause
                            } else {
                                self.state = State::Run
                            }
                        }
                        _ => {
                            for player in self.players.iter_mut() {
                                for key in player.keys.iter() {
                                    if key_event.code == KeyCode::Char(*key.1) {
                                        if let Some(snake) = player.snake.as_mut() {
                                            snake.direction = *key.0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if delta >= self.tick_rate {
                self.last_update = now;

                match self.state {
                    State::Init => self.init().unwrap(),
                    State::Run => self.run().unwrap(),
                    State::Pause => self.pause().unwrap(),
                    State::Stop => break,
                }
            }
        }

        crossterm::terminal::disable_raw_mode()?;
        execute!(self.out, cursor::Show)?;

        Ok(())
    }

    fn init(&mut self) -> io::Result<()> {
        queue!(self.out, terminal::Clear(terminal::ClearType::All)).unwrap();
        self.generate_players();
        self.generate_food();

        for player in self.players.iter() {
            if let Some(snake) = player.snake {
                self.spatial_grid.add_object(ObjectRef::Player(snake.id()), snake.positions());
            }
        }

        for food in self.foods.iter() {
            self.spatial_grid.add_object(ObjectRef::Food { obj_id: food.id(), elem_id: (), kind: (), meals: () }, positions);
        }

        self.state = State::Run;
        Ok(())
    }

    fn pause(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn run(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn generate_food(&mut self) {
        let missing_food = self.food_spawns - self.foods.len();

        for _ in 0..missing_food {
            let pos = self.spatial_grid.rng_empty_pos(None);
            self.foods.push(Food::rng_food(self.id_counter.next(), pos));
        }
    }

    fn generate_players(&mut self) {
        let num_players = self.players.len();
        if num_players == 0 {
            return;
        }

        let border = self.spatial_grid.border;
        let playable_width = self.spatial_grid.full_width.saturating_sub(border * 2);
        let playable_height = self.spatial_grid.full_height.saturating_sub(border * 2);

        let step_x = playable_width as f32 / (num_players as f32 + 1.0);
        let step_y = playable_height as f32 / (num_players as f32 + 1.0);

        for (index, player) in self.players.iter_mut().enumerate() {
            let relative_x = ((index + 1) as f32 * step_x).round() as u16;
            let relative_y = ((index + 1) as f32 * step_y).round() as u16;

            let final_pos = Position {
                x: border + relative_x,
                y: border + relative_y,
            };

            let clamped_pos = Position {
                x: final_pos
                    .x
                    .clamp(border, self.spatial_grid.full_width - 1 - border),
                y: final_pos
                    .y
                    .clamp(border, self.spatial_grid.full_height - 1 - border),
            };

            player.add_snake(clamped_pos, self.id_counter.next(), 2);
        }
    }

    fn draw(&self) {
        if State::Init == self.state {

        }
    }

    // TODO - add collision
    fn collision_check(&mut self) {}

    // TODO - add drawing
}

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
