use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::Weak;

use util::internal_mut_struct;

use crate::component::LogicComponent;
use crate::component::LogicComponentFn;

struct SceneImpl {
    game_objects: Vec<Arc<GameObject>>,
}

internal_mut_struct!(Scene, SceneImpl, this: Weak<Scene>);

impl Scene {
    pub fn new() -> Arc<Self> {
        Arc::new_cyclic(|this| Self {
            inner: Mutex::new(SceneImpl {
                game_objects: Vec::new(),
            }),
            this: this.clone(),
        })
    }

    pub fn add_game_object(&self) -> Arc<GameObject> {
        let game_object = GameObject::new(self.this.clone());
        self.lock_inner().game_objects.push(game_object.clone());

        game_object
    }

    pub fn game_objects(&self) -> Vec<Arc<GameObject>> {
        self.lock_inner().game_objects.to_vec()
    }
}

struct GameObjectImpl {
    logic_component: Option<Arc<LogicComponent>>,
}

internal_mut_struct!(
    GameObject,
    GameObjectImpl,
    this: Weak<GameObject>,
    scene: Weak<Scene>
);

impl GameObject {
    fn new(scene: Weak<Scene>) -> Arc<Self> {
        Arc::new_cyclic(|this| Self {
            inner: Mutex::new(GameObjectImpl {
                logic_component: None,
            }),
            this: this.clone(),
            scene,
        })
    }

    pub fn add_logic_component<T>(&self, fun: T)
    where
        T: LogicComponentFn,
    {
        self.lock_inner().logic_component = Some(LogicComponent::new(self.this.clone(), fun));
    }

    pub fn remove_logic_component(&self) {
        self.lock_inner().logic_component = None;
    }

    pub fn logic_component(&self) -> Option<Arc<LogicComponent>> {
        self.lock_inner().logic_component.clone()
    }
}
