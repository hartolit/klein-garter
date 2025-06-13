mod level;
mod player;
mod global;

use std::{io::{self, Read}, time::{Duration, Instant}};
use crossterm::{cursor::MoveTo, style::{Print, SetBackgroundColor, SetForegroundColor}, QueueableCommand};
use player::{Direction, Snake, Player, PlayerKind};
use io::{stdout, Stdout, Write};
use level::Level;
use global::Position;

pub enum State {
    Init,
    Run,
    Stop,
    Pause,
}

pub enum GameMode {
    Local,
    Online,
}

pub struct Game {
    pub state: State,
    pub kind: GameMode,
    pub level: Level,
    pub players: Vec<Player>,
    pub stdout: Stdout,
    pub tick_rate: Duration,
    pub last_update: Instant,
}

impl Game {
    pub fn new(kind: GameMode, player_count: u16) -> Self {
        let level = Level::new(40, 20);
        let players: Vec<Player> = Vec::new();

        match kind {
            GameMode::Local => {
                for _ in 0..player_count {
                    
                }
                let player_pos = self.level.rng_pos(Some(4));
                self.players.push(Player::new(PlayerKind::Local, player_pos));
            },
            GameMode::Online => {
                
            }
        }

        Game { 
            state: State::Init,
            kind: kind,
            level: level, 
            players: vec![], 
            stdout: stdout(),
            tick_rate: Duration::new(0, 500),
            last_update: Instant::now(),
        }
    }

    pub fn start(&mut self) {
        print!("\x1B[?25l"); // TODO - Fix() Removes cursor

        match self.state {
            State::Init => self.init(),
            State::Run => self.run(),
            State::Pause => self.pause(),
            State::Stop => self.stop(),
        }
    }

    fn init(&mut self) {
        self.level.generate(&mut self.stdout).unwrap();

        match self.kind {
            GameMode::Local => {
                let player_pos = self.level.rng_pos(Some(4));
                self.players.push(Player::new(PlayerKind::Local, player_pos));
            },
            GameMode::Online => {

            }
        }

        self.state = State::Run;
    }
    
    fn pause (&mut self) {
        
    }

    fn run(&mut self) {

    }

    fn stop(&mut self) {

    }

    fn generate_players(&mut self, player_count: u16) {
        let offset: Position = Position { 
            x: (self.level.total_width().div_ceil(player_count)), 
            y: (self.level.total_height().div_ceil(player_count)) 
        };

        let mut curr_pos = offset;

        for _ in 0..player_count{
            self.players.push(Player::new(PlayerKind::Local, Position { 
                x: curr_pos.x, 
                y: curr_pos.y
            }));

            if curr_pos.x + offset.x >= self.level.total_width(){
                curr_pos.x = offset.x;
                curr_pos.y += offset.y;
            } else {
                curr_pos.x += offset.x;
            }
        }
    }
}









pub fn start() {
    print!("\x1B[?25l"); // Removes cursor
    
    let mut level = Level::new(40, 20);
    let mut player = Player::new(level.rng_pos(Some(2)));
    let mut stdout = stdout();

    level.generate(&mut stdout).unwrap();

    let mut buf = [0u8; 1];
    while player.snake.is_alive {
        io::stdin().read_exact(&mut buf).expect("Failed to read input");

        let input = buf[0] as char;

        match input {
            'w' => player.snake.direction = Direction::Up,
            's' => player.snake.direction = Direction::Down,
            'd' => player.snake.direction = Direction::Right,
            'a' => player.snake.direction = Direction::Left,
            _ => {}
        }
        
        player.snake.slither();

        collision_check(&mut player.snake, &mut level);
        draw(&mut player.snake, &level, &mut stdout).unwrap();
        stdout.flush().unwrap();
    }
}

fn collision_check(snake: &mut Snake, level: &mut Level) {
    if snake.head_pos.x < level.border_width
        || snake.head_pos.x > level.total_width() - level.border_width - 1
        || snake.head_pos.y < level.border_height 
        || snake.head_pos.y > level.total_height() - level.border_height - 1 {
        
            snake.is_alive = false;
        return;
    }

    for part in &snake.body_vec {
        if snake.head_pos == *part {
            snake.is_alive = false;
        }
    }

    level.food_vec.retain(|food|{
        if snake.head_pos == food.pos {
            snake.meals += food.meals;
            false
        } else {
            true
        }
    });
}

fn draw(snake: &mut Snake, level: &Level, stdout: &mut Stdout) -> io::Result<()> {
    // Draw head
    if let Some(color_ref) = level.bg_color_range.get(snake.head_pos.y as usize) {
        stdout.queue(SetBackgroundColor(*color_ref))?;
    }

    stdout
        .queue(SetForegroundColor(snake.head_color))?
        .queue(MoveTo(snake.head_pos.x, snake.head_pos.y))?
        .queue(Print(snake.head))?;

    // Draw first_body_part
    if let Some(firt_body_part) = &snake.body_vec.front().copied() {
        if let Some(color_ref) = level.bg_color_range.get(firt_body_part.y as usize){
            stdout.queue(SetBackgroundColor(*color_ref))?;
        }

        stdout
            .queue(SetForegroundColor(snake.body_color))?
            .queue(MoveTo(firt_body_part.x, firt_body_part.y))?
            .queue(Print(&snake.body))?;
    }

    if snake.meals > 0 {
        snake.meals -= 1;
    } else if snake.meals < 0 {
        let parts_to_remove = snake.meals.abs() as u16;

        for _ in 0..parts_to_remove {
            if snake.body_vec.len() <= 1 {
                snake.is_alive = false;
                break;
            }

            if let Some(last_part) = snake.body_vec.pop_back() {
                if let Some(color_ref) = level.bg_color_range.get(last_part.y as usize){
                    stdout.queue(SetBackgroundColor(*color_ref))?;
                }
    
                stdout
                    .queue(MoveTo(last_part.x, last_part.y))?
                    .queue(Print(&level.background))?;
            }
        }
        snake.meals = 0;
    } else {
        if let Some(last_part) = snake.body_vec.pop_back() {
            if let Some(color_ref) = level.bg_color_range.get(last_part.y as usize){
                stdout.queue(SetBackgroundColor(*color_ref))?;
            }

            stdout
                .queue(MoveTo(last_part.x, last_part.y))?
                .queue(Print(&level.background))?;
        }
    }

    if level.food_vec.len() < level.max_food{
        let missing_food = level.food_vec.len() - level.max_food;

        for _ in 0..missing_food {
        }
    }

    snake.body_vec.push_front(snake.head_pos);

    Ok(())
}