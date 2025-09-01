use std::any::{Any, TypeId};
use std::collections::HashMap;
use crate::core::scene::Scene;

pub trait Event: 'static + Any {
    fn as_any(&self) -> &dyn Any;
}

pub trait EventHandler<E: Event> {
    fn handle_event(&mut self, event: &E, scene: &mut Scene);
}

pub struct EventManager {
    handlers: HashMap<TypeId, Vec<Box<dyn FnMut(&dyn Any, &mut Scene)>>>,
}

impl EventManager {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    pub fn register<E: Event, H: EventHandler<E> + 'static>(&mut self, mut handler: H) {
        let event_type_id = TypeId::of::<E>();
        let entry = self.handlers.entry(event_type_id).or_default();

        let callback = move |event_any: &dyn Any, scene: &mut Scene| {
            if let Some(event) = event_any.downcast_ref::<E>() {
                handler.handle_event(event, scene);
            }
        };

        entry.push(Box::new(callback));
    }

    pub fn dispatch(&mut self, scene: &mut Scene) {
        let events = std::mem::take(&mut scene.event_bus);
        for event in &events {
            let event_any = event.as_any();
            if let Some(handlers) = self.handlers.get_mut(&event_any.type_id()) {
                for handler_fn in handlers {
                    handler_fn(event_any, scene);
                }
            }
        }
    }
}