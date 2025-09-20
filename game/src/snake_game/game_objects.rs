pub mod bomb;
pub mod food;
pub mod snake;

pub use bomb::Bomb;
pub use food::Food;
pub use snake::Snake;

pub trait Consumable {
    fn get_meal(&self) -> u16;
}

pub trait Damaging {
    fn get_damage(&self) -> u16;
}
