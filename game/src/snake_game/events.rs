use engine::prelude::{Event, Id, Position};
use std::any::Any;

pub struct CollisionEvent {
    pub actor: Id,
    pub target: Id,
    pub pos: Position,
}

impl Event for CollisionEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct FoodEatenEvent {
    pub snake_id: Id,
    pub food_id: Id,
}

impl Event for FoodEatenEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}