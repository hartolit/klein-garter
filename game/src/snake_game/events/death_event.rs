use engine::prelude::{Id, Position, Event, EventHandler, Scene};

pub struct DeathEvent {
    pub actor: Id,
    pub pos: Position,
}

impl Event for DeathEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct DeathHandler;
impl EventHandler<DeathEvent> for DeathHandler {
    fn handle_event(&mut self, event: &DeathEvent, scene: &mut Scene) {
        scene.remove_object(&event.actor);
    }
}