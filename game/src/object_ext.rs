use engine::core::global::{Id, Position};
use engine::core::object::state::StateChange;

pub trait Consumable {
    fn get_meal(&self) -> i16;
    fn on_consumed(&self, hit_element_id: Id, pos: Position, recipient_id: Id) -> StateChange;
}

pub trait Damaging {
    fn get_damage(&self) -> i16;
    fn on_hit(&self, hit_element_id: Id, pos: Position, recipient_id: Id) -> StateChange;
}
