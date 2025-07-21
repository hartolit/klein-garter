///
/// ID
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id {
    pub value: u64,
}

impl Id {
    pub fn new(id: u64) -> Self {
        Id { value: id }
    }
}

#[derive(Debug, Clone)]
pub struct IdCounter {
    counter: Id,
}

impl IdCounter {
    pub fn new() -> Self {
        Self {
            counter: Id::new(0),
        }
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
        Self { x, y }
    }
}
