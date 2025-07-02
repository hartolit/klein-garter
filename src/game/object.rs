use crossterm::style::Color;
use super::grid::{CellKind, ObjectRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Glyph {
    pub fg_clr: Option<Color>,
    pub bg_clr: Option<Color>,
    pub symbol: char,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub style: Glyph,
    pub pos: Position,
}

impl Element {
    pub fn new (style: Glyph, pos: Option<Position>) -> Self {
        Element { 
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


pub struct Collision<'a> {
    pub pos: Position,
    pub kind: &'a CellKind,
    pub colliders: &'a [ObjectRef],
}

pub trait Object {
    fn id(&self) -> ObjectId;
    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_>;
    fn positions(&self) -> Box<dyn Iterator<Item = Position> + '_>;
}

pub struct StateChange {
    pub obj_id: ObjectId,
    pub old_pos: Position,
    pub new_element: Option<Element>,
}

impl StateChange {
    pub fn new(obj_id: ObjectId, old_pos: Position, new_element: Option<Element>) -> Self {
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