use ::engine::core::global::Id;
use uuid::Uuid;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Player {
    pub id: Uuid,
    pub score: u16,
    pub snake: Option<Id>,
}

impl Player {
    pub fn new() -> Self {
        Player {
            id: Uuid::new_v4(),
            score: 0,
            snake: None,
        }
    }

    pub fn set_snake(&mut self, snake_id: Id) {
        self.snake = Some(snake_id)
    }
}
