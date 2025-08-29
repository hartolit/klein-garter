use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

pub mod global;
pub mod grid;
pub mod object;
pub mod renderer;
pub mod scene;

use object::Action;
use renderer::Renderer;
use scene::{ObjectIndex, Scene};

use crate::core::grid::SpatialGrid;

pub struct Stage<K: Eq + Hash + Clone> {
    logic: Box<dyn Logic<K>>,
    scene: Box<Scene>,
}

impl<K: Eq + Hash + Clone> Stage<K> {
    pub fn new(logic: Box<dyn Logic<K>>, grid: SpatialGrid) -> Self {
        Self {
            logic,
            scene: Box::new(Scene::new(grid)),
        }
    }

    pub fn replace_scene(&mut self, scene: Box<Scene>) -> Box<Scene> {
        let old_scene = std::mem::replace(&mut self.scene, scene);
        old_scene
    }

    pub fn replace_logic(&mut self, logic: Box<dyn Logic<K>>) -> Box<dyn Logic<K>> {
        let old_logic = std::mem::replace(&mut self.logic, logic);
        old_logic
    }

    pub fn replace_stage(
        &mut self,
        logic: Box<dyn Logic<K>>,
        scene: Box<Scene>,
    ) -> (Box<dyn Logic<K>>, Box<Scene>) {
        let old_logic = std::mem::replace(&mut self.logic, logic);
        let old_scene = std::mem::replace(&mut self.scene, scene);
        (old_logic, old_scene)
    }
}

pub trait Logic<K: Eq + Hash + Clone> {
    fn process_actions(&mut self, scene: &mut Scene, actions: Vec<Action>);
    fn process_input(&mut self, scene: &mut Scene);
    fn setup(&mut self, scene: &mut Scene);
    fn update(&mut self, scene: &mut Scene) -> RuntimeCommand<K>;
    fn collect_old_stage(
        &mut self,
        _old_scene: Option<Box<Scene>>,
        _old_logic: Option<Box<dyn Logic<K>>>,
    ) {
    }
}

pub enum RuntimeCommand<K: Eq + Hash + Clone> {
    ReplaceScene(Box<Scene>),
    ReplaceLogic(Box<dyn Logic<K>>),
    ReplaceStage {
        scene: Box<Scene>,
        logic: Box<dyn Logic<K>>,
    },
    SwitchStage(K),
    SetTickRate(Duration),
    Kill,
    None,
}

enum ManagerDirective<K: Eq + Hash + Clone> {
    Switch(K),
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

    pub fn set_active_key(&mut self, key: K) {
        if !self.stages.contains_key(&key) {
            panic!("Attempted to switch to a non-existent stage key!");
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
                        self.set_active_key(new_key);
                    }
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

pub struct Runtime {
    pub tick_rate: Duration,
    last_update: Instant,
    renderer: Renderer,
}

impl Runtime {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            tick_rate,
            last_update: Instant::now(),
            renderer: Renderer::new(),
        }
    }

    fn run<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) -> ManagerDirective<K> {
        self.initialize(stage);
        self.last_update = Instant::now();

        loop {
            stage.logic.process_input(&mut stage.scene);

            let now = Instant::now();
            let delta = now.duration_since(self.last_update);

            if delta >= self.tick_rate {
                self.last_update = now;
                let command = stage.logic.update(&mut stage.scene);

                if let Some(directive) = self.execute_command(command, stage) {
                    return directive;
                }

                self.tick(stage);
                stage.scene.sync();
                self.renderer.partial_render(
                    &stage.scene.spatial_grid,
                    &stage.scene.global_state.finalized,
                );
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn initialize<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) {
        stage.logic.setup(&mut stage.scene);
        self.renderer
            .full_render(&mut stage.scene.spatial_grid, &stage.scene.objects);
    }

    fn tick<K: Eq + Hash + Clone>(&mut self, stage: &mut Stage<K>) {
        let future_moves = stage
            .scene
            .indexes
            .get(&ObjectIndex::Movable)
            .into_iter()
            .flat_map(|set| set.iter())
            .filter_map(|id| {
                stage
                    .scene
                    .objects
                    .get(id)
                    .and_then(|obj| obj.as_movable())
                    .map(|movable| (*id, movable))
            })
            .flat_map(|(id, movable)| movable.predict_pos().map(move |pos| (id, pos)));

        let mut probe_map = stage.scene.spatial_grid.probe_moves(future_moves);

        let mut actions: Vec<Action> = Vec::new();
        for (id, probe) in probe_map.drain() {
            if let Some(object) = stage.scene.objects.get_mut(&id) {
                if let Some(movable) = object.as_movable_mut() {
                    actions.extend(movable.make_move(probe));
                }
            }
        }

        stage.logic.process_actions(&mut stage.scene, actions);
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
            RuntimeCommand::ReplaceStage { scene, logic } => {
                let old_stage = stage.replace_stage(logic, scene);
                stage
                    .logic
                    .collect_old_stage(Some(old_stage.1), Some(old_stage.0));
            }
            RuntimeCommand::SwitchStage(key) => return Some(ManagerDirective::Switch(key)),
            RuntimeCommand::SetTickRate(tick_rate) => self.tick_rate = tick_rate,
            RuntimeCommand::Kill => return Some(ManagerDirective::Kill),
            RuntimeCommand::None => {}
        }
        None
    }
}