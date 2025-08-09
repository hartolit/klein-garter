use crate::core::object::state::{State, StateChange};

#[derive(Debug)]
pub struct GlobalState {
    pub state: State,
    pub finalized: CategorizedStates,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            state: State::new(),
            finalized: CategorizedStates::new(),
        }
    }

    pub fn finalize(&mut self) {
        self.finalized.clear();

        for (_, change) in self.state.changes.drain() {
            match change {
                StateChange::Create { .. } => self.finalized.creates.push(change),
                StateChange::Delete { .. } => self.finalized.deletes.push(change),
                StateChange::Update { .. } => self.finalized.updates.push(change),
            }
        }
    }
}

#[derive(Debug)]
pub struct CategorizedStates {
    pub creates: Vec<StateChange>,
    pub deletes: Vec<StateChange>,
    pub updates: Vec<StateChange>,
}

impl CategorizedStates {
    pub fn new() -> Self {
        Self {
            creates: Vec::new(),
            deletes: Vec::new(),
            updates: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.creates.clear();
        self.deletes.clear();
        self.updates.clear();
    }
}
