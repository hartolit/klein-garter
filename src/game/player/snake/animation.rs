use crossterm::style::Color;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum EffectStyle {
    Grow,
    Damage,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum EffectZone {
    Head,
    Body,
    Tail,
    All,
}

impl EffectStyle {
    fn color(&self) -> Option<Color> {
        match self {
            EffectStyle::Damage => Some(Color::Red),
            EffectStyle::Grow => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Effect {
    duration: usize,
    pub kind: EffectStyle,
    pub zone: EffectZone,
    pub action_size: Option<usize>,
}

impl Effect {
    pub fn new(duration: usize, kind: EffectStyle, action_size: Option<usize>, zone: EffectZone) -> Self {
        Self { duration, kind, action_size, zone }
    }

    pub fn next_tick(&mut self) {
        self.duration = self.duration.saturating_sub(1);
    }
}

