use engine::prelude::{Event, EventHandler, Id, ObjectExt, Scene};

use crate::snake_game::game_objects::{
    snake::animation::{Effect, EffectStyle, EffectZone},
    {Bomb, Damaging, Snake},
};

pub struct BombEvent {
    pub snake_id: Id,
    pub bomb_id: Id,
}

impl Event for BombEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn log_message(&self) -> String {
        format!(
            "[BOMB]: A:{}, T:{}",
            self.snake_id.value, self.bomb_id.value,
        )
    }
}

pub struct BombHandler;
impl EventHandler<BombEvent> for BombHandler {
    fn handle_event(&mut self, event: &BombEvent, scene: &mut Scene) {
        let mut damage = 0;
        if let Some(object) = scene.objects.get(&event.bomb_id) {
            if let Some(bomb) = object.get::<Bomb>() {
                damage = bomb.get_damage();
            }
        }

        if damage != 0 {
            if let Some(snake) = scene
                .objects
                .get_mut(&event.snake_id)
                .and_then(|obj| obj.get_mut::<Snake>())
            {
                snake.meals = snake.meals.saturating_sub(damage as i16);
                snake.apply_effect(Effect::new(
                    5 + damage as usize,
                    EffectStyle::Damage,
                    Some(snake.head_size.native_size().saturating_add(2)),
                    EffectZone::All,
                ));
            }
        }

        scene.remove_object(&event.bomb_id);
    }
}
