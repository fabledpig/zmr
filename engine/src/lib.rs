use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use scene::Scene;
use util::{internal_mut_struct, job::Scheduler, logger::LoggerClient, thread_category};

pub mod component;
pub mod scene;

thread_category!(EngineThreadCategory, Logger, GameObject);

struct EngineContextImpl {
    scene: Arc<Scene>,
}

impl EngineContextImpl {
    fn new(scene: Arc<Scene>) -> Self {
        Self { scene }
    }
}

internal_mut_struct!(
    EngineContext,
    EngineContextImpl,
    logger_client: LoggerClient,
    scheduler: Scheduler<EngineThreadCategory>
);

impl EngineContext {
    pub fn new(
        logger_client: LoggerClient,
        scheduler: Scheduler<EngineThreadCategory>,
        scene: Arc<Scene>,
    ) -> Self {
        Self {
            logger_client,
            scheduler,
            inner: Mutex::new(EngineContextImpl::new(scene)),
        }
    }

    pub fn logger_client(&self) -> &LoggerClient {
        &self.logger_client
    }

    pub fn scheduler(&self) -> &Scheduler<EngineThreadCategory> {
        &self.scheduler
    }

    pub fn scene(&self) -> Arc<Scene> {
        self.lock_inner().scene.clone()
    }
}

pub struct Engine {
    engine_context: EngineContext,
}

impl Engine {
    pub fn new(
        scheduler: Scheduler<EngineThreadCategory>,
        logger_client: LoggerClient,
        scene: Arc<Scene>,
    ) -> Self {
        Self {
            engine_context: EngineContext::new(logger_client, scheduler, scene),
        }
    }

    pub fn update(&self, _delta_time: Duration) {
        let scene = self.engine_context.scene();
        self.engine_context.scheduler().scoped(|s| {
            for game_object in scene.game_objects() {
                if let Some(logic_component) = game_object.logic_component() {
                    s.schedule_job(EngineThreadCategory::GameObject, move || {
                        logic_component.run(&self.engine_context);
                    });
                }
            }
        });
    }
}
