use std::sync::{Arc, Mutex, MutexGuard};

use util::{
    holder_ref::{HolderRef, WithHolderRef},
    internal_mut_struct,
};

use crate::component::{LogicComponent, LogicComponentFn};

struct SceneImpl {
    game_objects: Vec<Arc<GameObject>>,
}

internal_mut_struct!(Scene, SceneImpl);

impl Scene {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(SceneImpl {
                game_objects: Vec::new(),
            }),
        }
    }

    pub fn add_game_object(&self) -> WithHolderRef<GameObject> {
        let game_object = GameObject::new();
        self.lock_inner().game_objects.push(game_object.clone());

        unsafe { WithHolderRef::new(self, game_object) }
    }

    pub fn game_objects(&self) -> Vec<WithHolderRef<GameObject>> {
        self.lock_inner()
            .game_objects
            .iter()
            .map(|game_object| unsafe { WithHolderRef::new(self, game_object.clone()) })
            .collect()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

struct GameObjectImpl {
    scene: Option<&'static Scene>,
    logic_component: Option<Arc<LogicComponent>>,
}

internal_mut_struct!(GameObject, GameObjectImpl);

impl GameObject {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(GameObjectImpl {
                scene: None,
                logic_component: None,
            }),
        })
    }

    pub fn add_logic_component<T>(&self, fun: T)
    where
        T: LogicComponentFn,
    {
        self.lock_inner().logic_component = Some(LogicComponent::new(fun));
    }

    pub fn remove_logic_component(&self) {
        self.lock_inner().logic_component = None;
    }

    pub fn logic_component(&self) -> Option<WithHolderRef<LogicComponent>> {
        self.lock_inner()
            .logic_component
            .as_ref()
            .map(|logic_component| unsafe { WithHolderRef::new(self, logic_component.clone()) })
    }
}

impl HolderRef for GameObject {
    type HolderType = Scene;

    fn set_holder(&self, holder_ref: &'static Self::HolderType) {
        self.lock_inner().scene = Some(holder_ref);
    }
}
