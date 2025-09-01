use engine::core::scene::{Scene};
use engine::prelude::{EventHandler, ObjectExt};

use crate::snake_game::events::{CollisionEvent, FoodEatenEvent};
use crate::snake_game::game_object::Consumable;
use crate::snake_game::{Snake, Food};
pub struct CollisionHandler;

impl EventHandler<CollisionEvent> for CollisionHandler {
    fn handle_event(&mut self, event: &CollisionEvent, scene: &mut Scene) {
        let actor = scene.objects.get(&event.actor).and_then(|obj| obj.get::<Snake>()).is_some();
        let target = scene.objects.get(&event.target).and_then(|obj| obj.get::<Food>()).is_some();

        if actor && target {
            scene.event_bus.push(Box::new(FoodEatenEvent {
                snake_id: event.actor,
                food_id: event.target,
            }));
        }
    }
}

pub struct FoodEatenHandler;
impl EventHandler<FoodEatenEvent> for FoodEatenHandler {
    fn handle_event(&mut self, event: &FoodEatenEvent, scene: &mut Scene) {
        let mut meals = 0;
        if let Some(object) = scene.objects.get(&event.food_id) {
            if let Some(food) = object.get::<Food>() {
                meals = food.get_meal();
            }
        }

        if meals != 0 {
            if let Some(snake) = scene.objects.get_mut(&event.snake_id).and_then(|obj| obj.get_mut::<Snake>()) {
                snake.meals += meals;
            }
        }

        scene.remove_object(&event.food_id);
    }
}