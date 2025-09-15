use engine::prelude::{Event, EventHandler, Id, ObjectExt, Scene};

use crate::snake_game::food::Food;
use crate::snake_game::game_object::Consumable;
use crate::snake_game::snake::Snake;
use crate::snake_game::snake::animation::{Effect, EffectStyle, EffectZone};

pub struct FoodEatenEvent {
    pub snake_id: Id,
    pub food_id: Id,
}

impl Event for FoodEatenEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn log_message(&self) -> String {
        format!(
            "[FOOD EATEN]: A:{}, T:{}",
            self.snake_id.value, self.food_id.value,
        )
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
            if let Some(snake) = scene
                .objects
                .get_mut(&event.snake_id)
                .and_then(|obj| obj.get_mut::<Snake>())
            {
                snake.meals += meals;
                snake.apply_effect(Effect::new(
                    3,
                    EffectStyle::Grow,
                    Some(snake.head_size.native_size() + 2),
                    EffectZone::All,
                ));
            }
        }

        scene.remove_object(&event.food_id);
    }
}
