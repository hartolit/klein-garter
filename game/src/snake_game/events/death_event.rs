use engine::prelude::{Event, EventHandler, Id, Position, Scene};

pub struct DeathEvent {
    pub actor: Id,
    pub pos: Position,
}

impl Event for DeathEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn log_message(&self) -> String {
        format!(
            "[DEATH]: A:{} @({},{})",
            self.actor.value, self.pos.x, self.pos.y
        )
    }
}

pub struct DeathHandler;
impl EventHandler<DeathEvent> for DeathHandler {
    fn handle_event(&mut self, event: &DeathEvent, scene: &mut Scene) {
        scene.remove_object(&event.actor);
    }
}
