use engine::prelude::{Event, EventHandler, Id, ObjectExt, Scene};

use crate::snake_game::game_objects::{
    snake::animation::{Effect, EffectStyle, EffectZone},
    {Consumable, Food, Snake},
};

pub struct FoodEvent {
    pub snake_id: Id,
    pub food_id: Id,
}

impl Event for FoodEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn log_message(&self) -> String {
        format!(
            "[FOOD]: A:{}, T:{}",
            self.snake_id.value, self.food_id.value,
        )
    }
}

pub struct FoodHandler;
impl EventHandler<FoodEvent> for FoodHandler {
    fn handle_event(&mut self, event: &FoodEvent, scene: &mut Scene) {
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
                snake.meals = snake.meals.saturating_add(meals as i16);
                snake.apply_effect(Effect::new(
                    5 + meals as usize,
                    EffectStyle::Grow,
                    Some(snake.head_size.native_size().saturating_add(2)),
                    EffectZone::All,
                ));
            }
        }

        scene.remove_object(&event.food_id);
    }
}
