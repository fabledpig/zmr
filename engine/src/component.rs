use std::sync::{Arc, Mutex, MutexGuard};

use util::{holder_ref::HolderRef, internal_mut_struct};

use crate::{scene::GameObject, EngineContext};

pub trait LogicComponentFn: Fn(&EngineContext, &GameObject) + Send + Sync + 'static {}

impl<T> LogicComponentFn for T where T: Fn(&EngineContext, &GameObject) + Send + Sync + 'static {}

struct LogicComponentImpl {
    game_object: Option<&'static GameObject>,
    fun: Box<dyn LogicComponentFn>,
}

internal_mut_struct!(LogicComponent, LogicComponentImpl);

impl LogicComponent {
    pub fn new<T>(fun: T) -> Arc<Self>
    where
        T: LogicComponentFn,
    {
        Arc::new(Self {
            inner: Mutex::new(LogicComponentImpl {
                game_object: None,
                fun: Box::new(fun),
            }),
        })
    }

    pub fn run(&self, engine_context: &EngineContext) {
        let inner = self.lock_inner();
        (inner.fun)(engine_context, inner.game_object.unwrap());
    }
}

impl HolderRef for LogicComponent {
    type HolderType = GameObject;

    fn set_holder(&self, holder_ref: &'static Self::HolderType) {
        self.lock_inner().game_object = Some(holder_ref);
    }
}
