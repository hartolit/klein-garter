use engine::core::global::{Id, IdCounter, Position};
use engine::core::object::{
    Object, Stateful,
    t_cell::{TCell, Glyph},
    state::{State, StateChange},
};

#[derive(Debug)]
pub struct Button {
    pub id: Id,
    is_selected: bool,
    label: String,
    id_counter: IdCounter,
    elements: Vec<TCell>,
    state: State,
    center_pos: Position,
}

impl Button {
    pub fn new(obj_id: Id, center_pos: Position, label: String) -> Self {
        Self {
            id: obj_id,
            is_selected: false,
            label,
            id_counter: IdCounter::new(),
            elements: Vec::new(),
            state: State::new(),
            center_pos,
        }
    }
}

impl Object for Button {
    fn id(&self) -> Id {
        self.id
    }

    fn elements(&self) -> Box<dyn Iterator<Item = &engine::core::object::t_cell::TCell> + '_> {
        Box::new(self.elements.iter())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Stateful for Button {
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    fn state(&self) -> &State {
        &self.state
    }
}
