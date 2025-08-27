use std::any::Any;
use std::fmt::Debug;

pub mod state;
pub mod t_cell;

use super::global::{Id, Position};
use super::grid::cell::CellRef;
use crate::core::object::state::State;
use state::StateChange;
use t_cell::TCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Occupant {
    pub obj_id: Id,
    pub t_cell_id: Id,
}

impl Occupant {
    pub fn new(obj_id: Id, t_cell_id: Id) -> Self {
        Occupant { obj_id, t_cell_id }
    }
}

pub trait Object: Debug {
    fn id(&self) -> Id;
    fn t_cells(&self) -> Box<dyn Iterator<Item = &TCell> + '_>;

    // fn z_index(&self) -> i16 {
    //     0
    // } // TODO - Add z-index for object overlapping.

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_stateful(&self) -> Option<&dyn Stateful> {
        None
    }
    fn as_stateful_mut(&mut self) -> Option<&mut dyn Stateful> {
        None
    }
    fn as_movable(&self) -> Option<&dyn Movable> {
        None
    }
    fn as_movable_mut(&mut self) -> Option<&mut dyn Movable> {
        None
    }
    fn as_destructible(&self) -> Option<&dyn Destructible> {
        None
    }
    fn as_destructible_mut(&mut self) -> Option<&mut dyn Destructible> {
        None
    }
}

pub trait Movable {
    fn predict_pos(&self) -> Box<dyn Iterator<Item = Position> + '_>;
    fn make_move(&mut self, probe: Vec<CellRef>) -> Vec<Action>;
}

pub trait Stateful {
    fn state_mut(&mut self) -> &mut State;
    fn state(&self) -> &State;
    fn state_changes(&self) -> Box<dyn Iterator<Item = &StateChange> + '_> {
        Box::new(self.state().changes.values())
    }
}

pub trait Destructible: Object + Stateful {
    fn kill(&mut self) {
        let t_cell_data: Vec<_> = self.t_cells().map(|e| (e.occ, e.pos)).collect();
        let state_manager = self.state_mut();

        for (t_cell_occ, pos) in t_cell_data {
            state_manager.upsert_change(StateChange::Delete {
                occupant: t_cell_occ,
                init_pos: pos,
            });
        }
    }
}

pub trait ObjectExt {
    fn get<T: 'static>(&self) -> Option<&T>;
    fn get_mut<T: 'static>(&mut self) -> Option<&mut T>;
}

impl ObjectExt for dyn Object {
    fn get<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Action {
    Collision { owner: Occupant, target: Occupant },
    Kill { obj_id: Id },
}
