use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

pub mod event;
pub mod global;
pub mod runtime;

use runtime::Runtime;

use crate::prelude::Stage;

pub enum ManagerDirective<K: Eq + Hash + Clone> {
    Switch(K),
    Refresh,
    Kill,
}

pub struct RuntimeManager<K: Eq + Hash + Clone> {
    runtime: Runtime,
    stages: HashMap<K, Stage<K>>,
    active_key: Option<K>,
}

impl<K: Eq + Hash + Clone> RuntimeManager<K> {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            runtime: Runtime::new(tick_rate),
            stages: HashMap::new(),
            active_key: None,
        }
    }

    pub fn add_stage(&mut self, key: K, stage: Stage<K>) {
        self.stages.insert(key, stage);
    }

    pub fn set_active_stage(&mut self, key: K) {
        if !self.stages.contains_key(&key) {
            panic!("Attempted to set stage with a non-existent key!");
        }
        self.active_key = Some(key);
    }

    pub fn run_app(&mut self) {
        loop {
            if let Some(active_key) = self.active_key.clone() {
                let mut active_stage = self
                    .stages
                    .remove(&active_key)
                    .expect("Active stage does not exists!");

                let directive = self.runtime.run(&mut active_stage);

                self.stages.insert(active_key, active_stage);

                match directive {
                    ManagerDirective::Switch(new_key) => {
                        self.set_active_stage(new_key);
                    }
                    ManagerDirective::Refresh => continue,
                    ManagerDirective::Kill => {
                        self.runtime.renderer.kill();
                        break;
                    }
                }
            } else {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
