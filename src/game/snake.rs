use std::collections::{VecDeque};

use super::level::Level;
use super::utils::Position;
use super::terminal::{Renderer, Rgb};

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Snake {
    pub is_alive: bool,
    pub meals: u16,
    pub head: char,
    pub body: char,
    pub head_pos: Position,
    pub body_vec: VecDeque<Position>,
    pub head_color: Rgb,
    pub body_color: Rgb,
    pub direction: Direction,
}

impl Snake {
    pub fn new(level: &Level) -> Self {
        // Align start pos to center
        let y = (level.height + level.border_height).div_ceil(2);
        let mut x = (level.width + level.border_width * 2).div_ceil(2);
        if x % 2 != 0 {
            x -= 1;
        }
        
        return Self {
            is_alive: true,
            meals: 10,
            head: '\u{2588}',
            body: '\u{25FC}',
            head_pos: Position { x: x, y: y },
            body_vec: VecDeque::new(),
            head_color: Rgb(50, 80, 120),
            body_color: Rgb(200, 200, 200),
            direction: Direction::Down,
        };
    }

    pub fn slither(&mut self, level: &mut Level, render: &mut Renderer) {
        match self.direction {
            Direction::Up => self.head_pos.y -= 1,
            Direction::Down => self.head_pos.y += 1,
            Direction::Left => self.head_pos.x -= 1,
            Direction::Right => self.head_pos.x += 1,
        };

        self.collision_check(level);
        self.draw(level, render);
    }

    fn collision_check(&mut self, level: &mut Level) {
        if self.head_pos.x < level.border_width
            || self.head_pos.x > level.total_width() - level.border_width - 1
            || self.head_pos.y < level.border_height 
            || self.head_pos.y > level.total_height() - level.border_height - 1 {
            
                self.is_alive = false;
            return;
        }

        for part in &self.body_vec {
            if self.head_pos == *part {
                self.is_alive = false;
            }
        }

        // level.food_vec.retain(|food|{
        //     if snake.head_pos == *food.pos() {
        //         snake.meals += food.meals();
        //         false
        //     } else {
        //         true
        //     }
        // });

        for food in &level.food_vec {
            if self.head_pos == *food.pos() {
                self.meals += food.meals();
            }
        }
    }

    fn draw(&mut self, level: &Level, render: &mut Renderer) {
        render.set_fg(&self.head_color);
        render.set_cursor(&self.head_pos);
        render.write(&self.head);

        // Draw first body
        if let Some(firt_part) = &self.body_vec.front().copied() {
            if let Some(color_ref) = level.bg_color_range.get(firt_part.y as usize){
                render.set_bg(color_ref);
            }
            render.set_fg(&self.body_color);
            render.set_cursor(firt_part);
            render.write(&self.body);
        }

        if self.meals > 0 {
            self.meals -= 1;
        } else {
            if let Some(last_part) = &self.body_vec.pop_back() {
                if let Some(color_ref) = level.bg_color_range.get(last_part.y as usize){
                    render.set_bg(color_ref);
                }
                render.set_cursor(last_part);
                render.write(&level.background);
            }
        }

        self.body_vec.push_front(self.head_pos);

        render.flush();
    }
}