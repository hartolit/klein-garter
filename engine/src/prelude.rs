pub use crate::core::global::{Id, IdCounter, Position};
pub use crate::core::grid::cell::{CellRef, Kind};
pub use crate::core::object::{
    Action, Destructible, Movable, Object, Occupant, Stateful,
    state::{State, StateChange},
    t_cell::{Glyph, TCell},
};

pub use crate::define_object;