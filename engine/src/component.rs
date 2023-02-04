use std::sync::{Arc, Weak};

use crate::{scene::GameObject, EngineContext};

pub trait LogicComponentFn: Fn(&EngineContext, &GameObject) + Send + Sync + 'static {}

impl<T> LogicComponentFn for T where T: Fn(&EngineContext, &GameObject) + Send + Sync + 'static {}

pub struct LogicComponent {
    game_object: Weak<GameObject>,
    fun: Box<dyn LogicComponentFn>,
}

impl LogicComponent {
    pub fn new<T>(game_object: Weak<GameObject>, fun: T) -> Arc<Self>
    where
        T: LogicComponentFn,
    {
        Arc::new(Self {
            game_object,
            fun: Box::new(fun),
        })
    }

    pub fn run(&self, engine_context: &EngineContext) {
        if let Some(game_object) = self.game_object.upgrade() {
            (self.fun)(engine_context, &game_object);
        }
    }
}
