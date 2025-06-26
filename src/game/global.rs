use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u32);

#[derive(Debug, Clone, Copy, Hash)]
pub struct Element {
    pub fg_clr: Color,
    pub bg_clr: Color,
    pub symbol: char,
    pub pos: Position,
}

pub struct StateChange {
    pub object_id: ObjectId,
    pub old_pos: Vec<Position>,
    pub new_elements: Vec<Element>,
}

pub struct Collision<'a> {
    pub pos: Position,
    pub colliders: &'a [ObjectRef],
}

pub trait Object {
    fn elements(&self) -> Box<dyn Iterator<Item = &Element> + '_>;
    fn positions(&self) -> Box<dyn Iterator<Item = &Position> + '_>;
    fn update(&mut self, collision: Option<Collision>) -> Option<StateChange>;
}

pub enum ObjectRef {
    Player(ObjectId),
    Food(ObjectId),
}

impl PartialEq for ObjectRef {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjectRef::Player(id1), ObjectRef::Player(id2)) => id1 == id2,
            (ObjectRef::Food(id1), ObjectRef::Food(id2)) => id1 == id2,
            _ => false,
        }
    }
}

impl Eq for ObjectRef {}

pub enum CellKind {
    Ground,
    Water,
    Lava,
    Border,
}

pub struct GridCell {
    pub occ_by: Vec<ObjectRef>,
    pub kind: CellKind
}

impl GridCell {
}

pub struct SpatialGrid {
    cells: Vec<GridCell>,
    pub width: u16,
    pub height: u16,
}