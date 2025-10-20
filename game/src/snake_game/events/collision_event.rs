use engine::prelude::{Event, EventHandler, Id, ObjectExt, Position, Scene};

use super::{BombEvent, FoodEvent};

use crate::snake_game::events::DeathEvent;
use crate::snake_game::game_objects::{Bomb, Food, Snake};

pub struct CollisionEvent {
    pub actor: Id,
    pub target: Id,
    pub pos: Position,
    pub ignore: bool, // Flag used to ignore certain operations like death
}

impl Event for CollisionEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn log_message(&self) -> String {
        format!(
            "[COLLISION]: A:{}, T:{} @({},{})",
            self.actor.value, self.target.value, self.pos.x, self.pos.y
        )
    }
}

pub struct CollisionHandler;

impl EventHandler<CollisionEvent> for CollisionHandler {
    fn handle_event(&mut self, event: &CollisionEvent, scene: &mut Scene) {
        #[derive(PartialEq, Eq)]
        enum ObjectType {
            Snake,
            Food,
            Bomb,
            Other,
            None,
        }

        let get_object_type = |id: &Id| {
            if let Some(obj) = scene.objects.get(id) {
                if obj.get::<Snake>().is_some() {
                    ObjectType::Snake
                } else if obj.get::<Food>().is_some() {
                    ObjectType::Food
                } else if obj.get::<Bomb>().is_some() {
                    ObjectType::Bomb
                } else {
                    ObjectType::Other
                }
            } else {
                ObjectType::None
            }
        };

        let actor_type = get_object_type(&event.actor);
        let target_type = get_object_type(&event.target);

        match (actor_type, target_type) {
            // Snake & Snake
            (ObjectType::Snake, ObjectType::Snake) => {
                // Prevents death of important or ignored objects
                if !scene.protected_ids.contains(&event.actor) && !event.ignore {
                    scene.event_bus.push(Box::new(DeathEvent {
                        actor: event.actor,
                        pos: event.pos,
                    }));
                }
            }

            // Snake & Food
            (ObjectType::Snake, ObjectType::Food) => {
                scene.event_bus.push(Box::new(FoodEvent {
                    snake_id: event.actor,
                    food_id: event.target,
                }));
            }
            (ObjectType::Food, ObjectType::Snake) => {
                scene.event_bus.push(Box::new(FoodEvent {
                    snake_id: event.target,
                    food_id: event.actor,
                }));
            }

            // Snake & Bomb
            (ObjectType::Snake, ObjectType::Bomb) => {
                scene.event_bus.push(Box::new(BombEvent {
                    snake_id: event.actor,
                    bomb_id: event.target,
                }));
            }
            (ObjectType::Bomb, ObjectType::Snake) => {
                scene.event_bus.push(Box::new(BombEvent {
                    snake_id: event.target,
                    bomb_id: event.actor,
                }));
            }
            _ => {}
        }
    }
}
