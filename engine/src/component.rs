use std::sync::{Arc, Mutex, MutexGuard};

use util::{holder_ref::HolderRef, internal_mut_struct};

use crate::scene::GameObject;

pub type LogicComponentFn = dyn Fn(&GameObject) + Send + Sync + 'static;

struct LogicComponentImpl {
    game_object: Option<&'static GameObject>,
    fun: Box<LogicComponentFn>,
}

internal_mut_struct!(LogicComponent, LogicComponentImpl);

impl LogicComponent {
    pub fn new<T>(fun: T) -> Arc<Self>
    where
        T: Fn(&GameObject) + Send + Sync + 'static,
    {
        Arc::new(Self {
            inner: Mutex::new(LogicComponentImpl {
                game_object: None,
                fun: Box::new(fun),
            }),
        })
    }

    pub fn run(&self) {
        let inner = self.lock_inner();
        (inner.fun)(inner.game_object.unwrap());
    }
}

impl HolderRef for LogicComponent {
    type HolderType = GameObject;

    fn set_holder(&self, holder_ref: Option<&'static Self::HolderType>) {
        self.lock_inner().game_object = holder_ref;
    }
}
