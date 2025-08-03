use engine::core::global::{Id, Position};
use engine::core::object::state::StateChange;

pub trait Consumable {
    fn get_meal(&self) -> i16;
}

pub trait Damaging {
    fn get_damage(&self) -> i16;
}
