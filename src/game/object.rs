use crossterm::style::Color;
use super::grid::{CellKind, ObjectRef};

///
/// BodySegment
/// 
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BodySegment {
    pub orientation: Orientation,
    pub elements: Vec<Element>
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical
}



/// 
/// RESIZESTATE
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeState {
    Normal { size: usize },
    Brief { size: usize, native_size: usize },
}

impl ResizeState {
    pub fn size(&self) -> usize {
        match self {
            ResizeState::Normal { size } => *size,
            ResizeState::Brief { size, ..} => *size,
        }
    }

    pub fn native(&self) -> usize {
        match self {
            ResizeState::Normal { size } => *size,
            ResizeState::Brief { native_size, ..} => *native_size,
        }
    }
}



/// 
/// ID AND ID GENERATION
/// 
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id{ pub value: u64 }

impl Id {
    pub fn new(id: u64) -> Self {
        Id { value: id }
    }
}

#[derive(Debug, Clone)]
pub struct IdCounter {
    counter: Id
}

impl IdCounter {
    pub fn new() -> Self {
        Self { counter: Id::new(0) }
    }

    pub fn next(&mut self) -> Id {
        let id = self.counter.value;
        self.counter.value += 1;
        Id::new(id)
    }
}



/// 
/// POSITION
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
        }
    }
}



/// 
/// ELEMENT AND OBJECT TRAIT
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Glyph {
    pub fg_clr: Option<Color>,
    pub bg_clr: Option<Color>,
    pub symbol: char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct  Element {
    pub id: Id,
    pub style: Glyph,
    pub pos: Position,
}

impl Element {
    pub fn new (id: Id, style: Glyph, pos: Option<Position>) -> Self {
        Element { 
            id,
            style,
            pos: {
                match pos {
                    Some(pos) => pos,
                    None => Position { x: 0, y: 0 }
                }
            }
        }
    }
}

pub trait Object {
    fn id(&self) -> Id;
    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_>;
    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_>;
}



/// 
/// COLLISION AND STATECHANGE
///
pub struct Collision<'a> {
    pub pos: Position,
    pub kind: &'a CellKind,
    pub colliders: &'a [ObjectRef],
}

pub struct StateChange {
    pub obj_id: Id,
    pub old_pos: Option<Position>,
    pub new_element: Option<Element>,
}

impl StateChange {
    pub fn new(obj_id: Id, old_pos: Option<Position>, new_element: Option<Element>) -> Self {
        Self {
            obj_id,
            old_pos,
            new_element,
        }
    }
}

pub trait DynamicObject: Object {
    fn next_pos(&self) -> Box<dyn Iterator<Item = Position> + '_>;
    fn update(&mut self, collisions: Option<Vec<Collision>>) -> Option<Vec<StateChange>>;
}