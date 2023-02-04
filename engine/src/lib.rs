use std::{thread, time::Duration};

use scene::Scene;
use util::{
    job::Scheduler,
    logger::{LogSeverity, LoggerClient},
    thread_category,
};

pub mod component;
pub mod scene;

thread_category!(EngineThreadCategory, Logger, GameObject);

pub struct EngineContext {
    logger_client: LoggerClient,
    scheduler: Scheduler<EngineThreadCategory>,
}

impl EngineContext {
    pub fn new(logger_client: LoggerClient, scheduler: Scheduler<EngineThreadCategory>) -> Self {
        Self {
            logger_client,
            scheduler,
        }
    }

    pub fn logger_client(&self) -> &LoggerClient {
        &self.logger_client
    }

    pub fn scheduler(&self) -> &Scheduler<EngineThreadCategory> {
        &self.scheduler
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
            self.engine_context.scheduler().scoped(|s| {
                for game_object in scene.game_objects() {
                    s.schedule_job(EngineThreadCategory::GameObject, move || {
                        if let Some(logic_component) = game_object.logic_component() {
                            logic_component.run(&self.engine_context);
                        }
                    });
                }
            });

            thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
        }
    }
}
