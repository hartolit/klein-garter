use crossterm::style::Color;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
    Grow,
    Damage,
}

impl Kind {
    fn color(&self) -> Option<Color> {
        match self {
            Kind::Damage => Some(Color::Red),
            Kind::Grow => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Effect {
    duration: usize,
    kind: Kind,
    pub action_size: Option<usize>,
}

impl Effect {
    pub fn new(duration: usize, kind: Kind, action_size: Option<usize>) -> Self {
        Self { duration, kind, action_size }
    }

    pub fn next_tick(&mut self) {
        self.duration = self.duration.saturating_sub(1);
    }
}