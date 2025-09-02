use std::any::Any;
use std::fmt::Debug;

pub mod state;
pub mod t_cell;

use super::global::{Id, Position};
use super::grid::cell::CellRef;
use crate::core::event::Event;
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

pub trait Stateful {
    fn state_mut(&mut self) -> &mut State;
    fn state(&self) -> &State;
    fn state_changes(&self) -> Box<dyn Iterator<Item = &StateChange> + '_> {
        Box::new(self.state().changes.values())
    }
}

pub trait Movable: Object + Stateful {
    /// Detect collisions by probing an objects future positions.
    /// If the predicted move is "non-pure" and includes itself,
    /// the collision system will treat this overlap as a collision.
    fn predict_pos(&self) -> Box<dyn Iterator<Item = Position> + '_>;
    fn make_move(&mut self, probe: Vec<CellRef>) -> Vec<Box<dyn Event>>;
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

// TODO - Revisit
#[macro_export]
macro_rules! define_object {
    // --- Internal Rules for parsing capability traits ---
    (@as_trait_impls) => {};

    (@as_trait_impls Stateful { $($body:tt)* } $($tail:tt)*) => {
        fn as_stateful(&self) -> Option<&dyn $crate::core::object::Stateful> { Some(self) }
        fn as_stateful_mut(&mut self) -> Option<&mut dyn $crate::core::object::Stateful> { Some(self) }
        $crate::define_object!(@as_trait_impls $($tail)*);
    };

    (@as_trait_impls Destructible { $($body:tt)* } $($tail:tt)*) => {
        fn as_destructible(&self) -> Option<&dyn $crate::core::object::Destructible> { Some(self) }
        fn as_destructible_mut(&mut self) -> Option<&mut dyn $crate::core::object::Destructible> { Some(self) }
        $crate::define_object!(@as_trait_impls $($tail)*);
    };

    (@as_trait_impls Movable { $($body:tt)* } $($tail:tt)*) => {
        fn as_movable(&self) -> Option<&dyn $crate::core::object::Movable> { Some(self) }
        fn as_movable_mut(&mut self) -> Option<&mut dyn $crate::core::object::Movable> { Some(self) }
        $crate::define_object!(@as_trait_impls $($tail)*);
    };

    (@trait_impls $struct:ty, ) => {};

    (@trait_impls $struct:ty, Stateful { state_field: $state_field:ident } $($tail:tt)*) => {
        impl $crate::core::object::Stateful for $struct {
            fn state(&self) -> &$crate::core::object::state::State { &self.$state_field }
            fn state_mut(&mut self) -> &mut $crate::core::object::state::State { &mut self.$state_field }
        }
        $crate::define_object!(@trait_impls $struct, $($tail)*);
    };

    (@trait_impls $struct:ty, Destructible { } $($tail:tt)*) => {
        impl $crate::core::object::Destructible for $struct {}
        $crate::define_object!(@trait_impls $struct, $($tail)*);
    };

    (@trait_impls $struct:ty, Movable { impl { $($body:tt)* } } $($tail:tt)*) => {
        impl $crate::core::object::Movable for $struct {
            $($body)*
        }
        $crate::define_object!(@trait_impls $struct, $($tail)*);
    };

    // --- Internal helper for t_cells logic ---
    (@expand_t_cells $self_expr:expr, single($body_field:ident)) => {
        Box::new(std::iter::once(&$self_expr.$body_field))
    };
    (@expand_t_cells $self_expr:expr, multi($body_field:ident)) => {
        Box::new($self_expr.$body_field.iter())
    };
    (@expand_t_cells $self_expr:expr, custom($func_name:ident)) => {
        $self_expr.$func_name()
    };

    // --- Public-Facing Rule ---
    (
        struct $struct:ty,
        id_field: $id_field:ident,
        t_cells: $kind:ident($($args:tt)*),
        capabilities: { $($capabilities:tt)* }
    ) => {
        impl $crate::core::object::Object for $struct {
            fn id(&self) -> $crate::core::global::Id { self.$id_field }
            fn t_cells(&self) -> Box<dyn Iterator<Item = &$crate::core::object::t_cell::TCell> + '_> {
                $crate::define_object!(@expand_t_cells self, $kind($($args)*))
            }
            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

            $crate::define_object!(@as_trait_impls $($capabilities)*);
        }

        $crate::define_object!(@trait_impls $struct, $($capabilities)*);
    };
}