mod terminal;
mod level;
mod snake;
mod utils;

use std::io::{self, Read};
use terminal::Renderer;

use snake::{Direction, Snake};
use level::Level;

pub fn start() {
    print!("\x1B[?25l"); // Removes cursor
    
    let mut level = Level::new(40, 20);
    let mut snake = Snake::new(&level);
    let mut render = Renderer::new();

    level.generate(&mut render);
    level.food_vec.push(level.rng_food());

    let mut buf = [0u8; 1];
    while snake.is_alive {
        io::stdin().read_exact(&mut buf).expect("Failed to read input");

        let input = buf[0] as char;

        match input {
            'w' => snake.direction = Direction::Up,
            's' => snake.direction = Direction::Down,
            'd' => snake.direction = Direction::Right,
            'a' => snake.direction = Direction::Left,
            _ => {}
        }
        
        snake.slither(&mut level, &mut render);

        render.flush();
    }
}

fn collision_check(){
    
}