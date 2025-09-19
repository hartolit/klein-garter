use super::object::state::{State, StateChange};

#[derive(Debug)]
pub struct GlobalState {
    pub state: State,
    pub filtered: CategorizedStates,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            state: State::new(),
            filtered: CategorizedStates::new(),
        }
    }

    pub fn process(&mut self, is_spatial: bool) {
        for (_, change) in self.state.changes.drain() {
            match is_spatial {
                true => self.filtered.spatial.push(change),
                false => self.filtered.non_spatial.push(change),
            }
        }

        match is_spatial {
            true => self.filtered.spatial.sort_by_key(|a| a.order()),
            false => self.filtered.non_spatial.sort_by_key(|a| a.order()),
        }
    }

    pub fn clear(&mut self) {
        self.state.clear();
        self.filtered.clear();
    }
}

#[derive(Debug)]
pub struct CategorizedStates {
    pub spatial: Vec<StateChange>,
    pub non_spatial: Vec<StateChange>,
}

impl CategorizedStates {
    pub fn new() -> Self {
        Self {
            spatial: Vec::new(),
            non_spatial: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.spatial.clear();
        self.non_spatial.clear();
    }
}
