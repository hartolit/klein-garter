use std::hash::Hash;
use std::time::{Duration, Instant};

pub mod renderer;
pub mod stage;

use super::ManagerDirective;
use crate::prelude::{Logic, ObjectIndex, Scene, Stage};
use renderer::Renderer;

pub enum RuntimeCommand<K: Eq + Hash + Clone> {
    ReplaceScene(Box<Scene>),
    ReplaceLogic(Box<dyn Logic<K>>),
    SwitchStage(K),
    SetTickRate(Duration),
    Reset,
    Skip,
    Kill,
    None,
}

pub struct Runtime {
    pub tick_rate: Duration,
    last_update: Instant,
    pub renderer: Renderer,
    skip_tick: bool,
}

impl Runtime {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            tick_rate,
            last_update: Instant::now(),
            renderer: Renderer::new(),
            skip_tick: false,
        }
    }

    pub fn run<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) -> ManagerDirective<K> {
        if !stage.is_init {
            self.initialize(stage);
        } else {
            self.refresh(stage);
        }

        self.last_update = Instant::now();
        loop {
            let now = Instant::now();
            let delta = now.duration_since(self.last_update);

            if delta >= self.tick_rate {
                self.last_update = now;
                let command = stage.logic.update(&mut stage.scene);
                if let Some(directive) = self.execute_command(command, stage) {
                    return directive;
                } else if self.skip_tick {
                    self.skip_tick = false;
                    continue;
                }

                self.tick(stage);
                stage.scene.sync();
                self.renderer.partial_render(&mut stage.scene);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn initialize<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) {
        stage.logic.setup(&mut stage.scene);
        stage.scene.sync();
        self.renderer.full_render(&mut stage.scene);
        stage.is_init = true;
    }

    fn refresh<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) {
        stage.scene.sync();
        self.renderer.full_render(&mut stage.scene);
    }

    fn tick<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) {
        // Gets events from movables (collisions)
        if let Some(grid) = &mut stage.scene.spatial_grid {            
            if let Some(movable_ids) = stage.scene.indexes.get(&ObjectIndex::Movable) {
                let future_moves = movable_ids
                    .iter()
                    .filter_map(|id| {
                        stage
                            .scene
                            .objects
                            .get(id)
                            .and_then(|obj| obj.as_movable())
                            .map(|movable| (*id, movable))
                    })
                    .flat_map(|(id, movable)| movable.probe_move().map(move |pos| (id, pos)));

                let mut probe_map = grid.probe_moves(future_moves);

                for id in movable_ids {
                    let probe = probe_map.remove(id);
                    if let Some(object) = stage.scene.objects.get_mut(id) {
                        if let Some(movable) = object.as_movable_mut() {
                            stage.scene.event_bus.extend(movable.make_move(probe));
                        }
                    }
                }
            }
        }

        // Gets events from active objects
        let active_events = stage
            .scene
            .indexes
            .get(&ObjectIndex::Active)
            .into_iter()
            .flat_map(|hash_set| hash_set.iter())
            .filter_map(|id| {
                stage
                    .scene
                    .objects
                    .get_mut(id)
                    .and_then(|obj| obj.as_active_mut())
                    .map(|active| active.update())
            })
            .flatten();

        stage.scene.event_bus.extend(active_events);

        stage.logic.dispatch_events(&mut stage.scene);
    }

    fn execute_command<K: Eq + Hash + Clone>(
        &mut self,
        command: RuntimeCommand<K>,
        stage: &mut Stage<K>,
    ) -> Option<ManagerDirective<K>> {
        match command {
            RuntimeCommand::ReplaceScene(scene) => {
                let old_scene = stage.replace_scene(scene);
                stage.logic.collect_old_stage(Some(old_scene), None);
            }
            RuntimeCommand::ReplaceLogic(logic) => {
                let old_logic = stage.replace_logic(logic);
                stage.logic.collect_old_stage(None, Some(old_logic));
            }
            RuntimeCommand::SwitchStage(key) => return Some(ManagerDirective::Switch(key)),
            RuntimeCommand::SetTickRate(tick_rate) => self.tick_rate = tick_rate,
            RuntimeCommand::Reset => {
                stage.scene.clear();
                stage.is_init = false;
                return Some(ManagerDirective::Refresh);
            }
            RuntimeCommand::Skip => self.skip_tick = true,
            RuntimeCommand::Kill => return Some(ManagerDirective::Kill),
            RuntimeCommand::None => {}
        }
        None
    }
}
