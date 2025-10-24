use crate::prelude::Scene;
use std::any::{Any, TypeId};
use rustc_hash::FxHashMap;

// TODO - Fix event duplications
// Self notes - (EventBus, EventKey, HashMap + Vec for O(1))

pub trait Event: 'static + Any {
    fn as_any(&self) -> &dyn Any;
    fn log_message(&self) -> String {
        format!("Event triggered!")
    }
}

pub trait EventHandler<E: Event> {
    fn handle_event(&mut self, event: &E, scene: &mut Scene);
}

pub struct EventManager {
    handlers: FxHashMap<TypeId, Vec<Box<dyn FnMut(&dyn Any, &mut Scene)>>>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            handlers: FxHashMap::default(),
        }
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
