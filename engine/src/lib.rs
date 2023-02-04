use std::{
    sync::{Mutex, MutexGuard},
    thread,
    time::Duration,
};

use scene::Scene;
use util::{
    internal_mut_struct,
    job::Scheduler,
    logger::{LogSeverity, LoggerClient},
    thread_category,
};

pub mod component;
pub mod scene;

thread_category!(EngineThreadCategory, Logger, GameObject);

struct EngineContextImpl {
    should_stop: bool,
}

impl EngineContextImpl {
    fn new() -> Self {
        Self { should_stop: false }
    }
}

internal_mut_struct!(
    EngineContext,
    EngineContextImpl,
    logger_client: LoggerClient,
    scheduler: Scheduler<EngineThreadCategory>
);

impl EngineContext {
    pub fn new(logger_client: LoggerClient, scheduler: Scheduler<EngineThreadCategory>) -> Self {
        Self {
            logger_client,
            scheduler,
            inner: Mutex::new(EngineContextImpl::new()),
        }
    }

    pub fn logger_client(&self) -> &LoggerClient {
        &self.logger_client
    }

    pub fn scheduler(&self) -> &Scheduler<EngineThreadCategory> {
        &self.scheduler
    }

    pub fn should_stop(&self) -> bool {
        self.lock_inner().should_stop
    }

    pub fn stop_engine(&self) {
        self.lock_inner().should_stop = true;
    }
}

pub struct Engine {
    engine_context: EngineContext,
}

impl Engine {
    pub fn new(scheduler: Scheduler<EngineThreadCategory>, logger_client: LoggerClient) -> Self {
        Self {
            engine_context: EngineContext::new(logger_client, scheduler),
        }
    }

    pub fn work(&self, initial_scene: &Scene) {
        self.engine_context
            .logger_client()
            .log(LogSeverity::Info, "Engine fired up");

        let scene = initial_scene;
        loop {
            if self.engine_context.should_stop() {
                break;
            }

            self.engine_context.scheduler().scoped(|s| {
                for game_object in scene.game_objects() {
                    if let Some(logic_component) = game_object.logic_component() {
                        s.schedule_job(EngineThreadCategory::GameObject, move || {
                            logic_component.run(&self.engine_context);
                        });
                    }
                }
            });

            thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
        }

        self.engine_context
            .logger_client()
            .log(LogSeverity::Info, "Engine stopped");
    }
}
