use std::any::Any;
use std::collections::HashMap;
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

/// The `Object` trait provides core functionality for objects inside the engine.
/// An object which only implements this trait only briefly announces its state
/// upon creation. After the breif state the object will remain static/silent
/// unless other object traits are added.
pub trait Object: Debug {
    fn id(&self) -> Id;
    fn t_cells(&self) -> Box<dyn Iterator<Item = &TCell> + '_>;

    /// Creates a brief creation state (used for a first render)
    fn init(&self) -> HashMap<Occupant, StateChange> {
        self.t_cells()
            .map(|t_cell| {
                let change = StateChange::Create {
                    new_t_cell: *t_cell,
                };
                (t_cell.occ, change)
            })
            .collect()
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_stateful(&self) -> Option<&dyn Stateful> {
        None
    }
    fn as_stateful_mut(&mut self) -> Option<&mut dyn Stateful> {
        None
    }
    fn as_destructible(&self) -> Option<&dyn Destructible> {
        None
    }
    fn as_destructible_mut(&mut self) -> Option<&mut dyn Destructible> {
        None
    }
    fn as_active(&self) -> Option<&dyn Active> {
        None
    }
    fn as_active_mut(&mut self) -> Option<&mut dyn Active> {
        None
    }
    fn as_spatial(&self) -> Option<&dyn Spatial> {
        None
    }
    fn as_spatial_mut(&mut self) -> Option<&mut dyn Spatial> {
        None
    }
    fn as_movable(&self) -> Option<&dyn Movable> {
        None
    }
    fn as_movable_mut(&mut self) -> Option<&mut dyn Movable> {
        None
    }
}

/// The `Stateful` trait is reactive.
/// The engine collects and renders states from a stateful object.
/// A stateful object would typically be triggered by an event through
/// an initiator (e.g. `Movable` or `Active` trait) or logic that changes
/// the state of the object. If there isn't an initiator of some kind,
/// a stateful object will remain non-reactive.
pub trait Stateful: Object {
    fn state_mut(&mut self) -> &mut State;
    fn state(&self) -> &State;
    fn state_changes(&self) -> Box<dyn Iterator<Item = &StateChange> + '_> {
        Box::new(self.state().changes.values())
    }
}

/// The `Destructible` trait creates a brief state to destroy an object.
/// The brief state enables static objects and objects which aren't
/// stateful, to be removed safely.
pub trait Destructible: Object {
    /// Creates a brief state for destruction of an object
    fn kill(&mut self) -> HashMap<Occupant, StateChange> {
        let mut kill_changes = HashMap::new();

        if let Some(stateful) = self.as_stateful_mut() {
            kill_changes.extend(stateful.state_mut().drain_changes());
        }

        let t_cells_kill = self.t_cells().map(|t_cell| {
            let change = StateChange::Delete {
                occupant: t_cell.occ,
                init_pos: t_cell.pos,
            };
            (t_cell.occ, change)
        });

        kill_changes.extend(t_cells_kill);
        kill_changes
    }
}

/// The `Active` trait is an initiator.
/// Objects with this trait can trigger events at each tick.
pub trait Active: Object {
    fn update(&mut self) -> Vec<Box<dyn Event>>;
}

/// Ties an object to a SpatialGrid
pub trait Spatial: Object {}

/// The `Movable` trait is a collision-based initiator.
/// The engine probes the object for future moves (`predict_pos`) and sends
/// collisions back to the object (`make_move`). The object then initiates
/// a move and sends back a reaction from its collisions through events.
pub trait Movable: Object + Stateful + Spatial {
    /// Probes an objects future positions to detect collisions.
    /// Note: If the predicted move is "non-pure" and includes itself,
    /// the collision system will treat this overlap as a collision.
    fn probe_move(&self) -> Box<dyn Iterator<Item = Position> + '_>;

    fn make_move(&mut self, probe: Vec<CellRef>) -> Vec<Box<dyn Event>>;
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

    (@as_trait_impls Spatial { $($body:tt)* } $($tail:tt)*) => {
        fn as_spatial(&self) -> Option<&dyn $crate::core::object::Spatial> { Some(self) }
        fn as_spatial_mut(&mut self) -> Option<&mut dyn $crate::core::object::Spatial> { Some(self) }
        $crate::define_object!(@as_trait_impls $($tail)*);
    };

    (@as_trait_impls Active { $($body:tt)* } $($tail:tt)*) => {
        fn as_active(&self) -> Option<&dyn $crate::core::object::Active> { Some(self) }
        fn as_active_mut(&mut self) -> Option<&mut dyn $crate::core::object::Active> { Some(self) }
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

    (@trait_impls $struct:ty, Spatial { } $($tail:tt)*) => {
        impl $crate::core::object::Spatial for $struct {}
        $crate::define_object!(@trait_impls $struct, $($tail)*);
    };

    (@trait_impls $struct:ty, Active { impl { $($body:tt)* } } $($tail:tt)*) => {
        impl $crate::core::object::Active for $struct {
            $($body)*
        }
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
