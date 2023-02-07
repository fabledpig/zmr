use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::time::Duration;

use input_handler::InputHandler;
use scene::Scene;
use util::internal_mut_struct;
use util::job::Scheduler;
use util::logger::LoggerClient;
use util::thread_category;

pub mod component;
pub mod input_handler;
pub mod renderer;
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
    scheduler: Scheduler<EngineThreadCategory>,
    input_handler: InputHandler
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
            input_handler: InputHandler::new(),
            inner: Mutex::new(EngineContextImpl::new(scene)),
        }
    }

    pub fn logger_client(&self) -> &LoggerClient {
        &self.logger_client
    }

    pub fn scheduler(&self) -> &Scheduler<EngineThreadCategory> {
        &self.scheduler
    }

    pub fn input_handler(&self) -> &InputHandler {
        &self.input_handler
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

    pub fn engine_context(&self) -> &EngineContext {
        &self.engine_context
    }

    pub fn update(&self, delta_time: Duration) {
        self.engine_context.input_handler().update(delta_time);
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
