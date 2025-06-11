mod level;
mod player;
mod global;

use std::io::{self, Read};
use crossterm::{cursor::MoveTo, style::{Print, SetBackgroundColor, SetForegroundColor}, QueueableCommand};
use player::{Direction, Snake, Player};
use io::{stdout, Stdout, Write};
use level::Level;

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
    if let Some(color_ref) = level.bg_color_range.get(snake.head_pos.y as usize){
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