use std::collections::VecDeque;
use rand::Rng;

use crate::game::utils::Position;

use super::terminal::{Renderer, Rgb};

#[derive(Debug)]
pub enum Food {
    Cherry { symbol: char, pos: Position, fg_color: Rgb },
    Mouse { symbol: char, pos: Position, fg_color: Rgb },
    Special { symbol: char, pos: Position, fg_color: Rgb },
}

impl Food {
    pub fn pos(&self) -> &Position {
        match self {
            Food::Cherry { pos, .. } |
            Food::Mouse { pos, .. } |
            Food::Special { pos, .. } => pos,
        }
    }

    pub fn meals(&self) -> u16 {
        match self {
            Food::Cherry { .. } => 1,
            Food::Mouse { .. } => 2,
            Food::Special { .. } => 5,
        }
    }
}

#[derive(Debug)]
pub struct Level {
    pub width: u16,
    pub height: u16,
    pub background: char,
    pub border: char,
    pub border_width: u16,
    pub border_height: u16,
    pub fg_color: Rgb,
    pub bg_color: Rgb,
    pub bg_color_range: Vec<Rgb>,
    pub food_vec: Vec<Food>
}

impl Level {
    pub fn new (mut width: u16, mut height: u16) -> Self {
        // Force even dimensions
        if width % 2 == 0 { width += 1; }
        if height % 2 == 0 { height += 1; }
        
        return  Self {
            width,
            height,
            background: ' ', // \u{2591}
            border: '\u{2588}',
            border_width: 2,
            border_height: 1,
            fg_color: Rgb(10, 100, 120),
            bg_color: Rgb(230, 40, 130),
            bg_color_range: vec![],
            food_vec: vec![],
        };
    }

    pub fn total_height(&self) -> u16 {
        self.height + self.border_height * 2
    }

    pub fn total_width(&self) -> u16 {
        self.width + self.border_width * 2
    }
    
    pub fn generate (&mut self, render: &mut Renderer) {
        render.clear_screen();
    
        // Generate bg_color_range
        {
            let mut tmp_green: u16 = self.bg_color.1 as u16;
            for _green in 0..self.total_height() {
                if !tmp_green + 10 <= 255{
                    self.bg_color_range.push(Rgb(self.bg_color.0, tmp_green as u8, self.bg_color.2));
                }else {
                    self.bg_color_range.push(Rgb(self.bg_color.0, tmp_green as u8, self.bg_color.2));
                    tmp_green += 10;
                }
            }
        }
        
        // Generate level
        for y in 0..self.total_height() {
            for x in 0..self.total_width() {
                // Set colors
                render.set_fg(&self.fg_color);
                if let Some(color_ref) = self.bg_color_range.get(y as usize){
                    render.set_bg(color_ref);
                }
    
                // Write border or background
                if x < self.border_width || x > self.width + self.border_width - 1 || y < self.border_height || y > self.height + self.border_height - 1 {
                    render.write(&self.border);
                } else {
                    render.write(&self.background);
                }
            }
            render.clear_styles();
            render.write('\n');
        }
    }

    pub fn rng_food(&self) -> Food {
        let rng_pos = self.rng_pos();

        match rand::rng().random_range(0..=2) {
            0 => Food::Cherry {
                symbol: 'ω',
                pos: rng_pos,
                fg_color: Rgb(255, 0, 0),
            },
            1 => Food::Mouse {
                symbol: 'Ω',
                pos: rng_pos,
                fg_color: Rgb(100, 100, 100),
            },
            _ => Food::Special {
                symbol: 'σ',
                pos: rng_pos,
                fg_color: Rgb(0, 255, 255),
            },
        }
    }

    pub fn rng_pos(&self) -> Position {
        let x = rand::rng().random_range(self.border_width..self.width + self.border_width);
        let y = rand::rng().random_range(self.border_height..self.height + self.border_height);
    
        Position { x: x, y: y }
    }
}


