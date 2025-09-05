use engine::{core::object::t_cell::TCell};

pub trait Consumable {
    fn get_meal(&self) -> i16;
}

pub trait Damaging {
    fn get_damage(&self) -> i16;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeState {
    Normal { size: usize },
    Brief { size: usize, native_size: usize },
}

impl ResizeState {
    pub fn current_size(&self) -> usize {
        match self {
            ResizeState::Normal { size } => *size,
            ResizeState::Brief { size, .. } => *size,
        }
    }

    pub fn native_size(&self) -> usize {
        match self {
            ResizeState::Normal { size } => *size,
            ResizeState::Brief { native_size, .. } => *native_size,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BodySegment {
    pub orientation: Orientation,
    pub t_cells: Vec<TCell>,
}

impl BodySegment {
    pub fn new(orientation: Orientation, t_cells: Vec<TCell>) -> Self {
        Self {
            orientation,
            t_cells,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
