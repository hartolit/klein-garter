use std::collections::HashMap;
use std::hash::Hash;

use rand::Rng;

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

pub struct SlotMap<T: Eq + Hash + Copy> {
    items: Vec<T>,
    map: HashMap<T, usize>,
}

impl<T: Eq + Hash + Copy> SlotMap<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn insert(&mut self, item: T) -> bool {
        if self.map.contains_key(&item) {
            return false;
        }

        let vec_index = self.items.len();
        self.items.push(item);
        self.map.insert(item, vec_index);
        true
    }

    pub fn remove(&mut self, item: &T) -> bool {
        if let Some(&vec_index) = self.map.get(item) {
            self.items.swap_remove(vec_index);

            if let Some(&swapped_item) = self.items.get(vec_index) {
                self.map.insert(swapped_item, vec_index);
            }

            self.map.remove(item);
            return true;
        }
        false
    }

    pub fn get_random(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let mut rng = rand::rng();
        let random_index = rng.random_range(0..self.items.len());
        Some(self.items[random_index])
    }
}
