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

pub struct Engine {
    logger_client: LoggerClient,
    scheduler: Scheduler<EngineThreadCategory>,
}

impl Engine {
    pub fn new(scheduler: Scheduler<EngineThreadCategory>, logger_client: LoggerClient) -> Self {
        Self {
            logger_client,
            scheduler,
        }
    }

    pub fn work(&self, initial_scene: &Scene) {
        self.logger_client.log(LogSeverity::Info, "Engine fired up");

        let scene = initial_scene;
        loop {
            for game_object in scene.game_objects() {
                if let Some(logic_component) = game_object.logic_component() {
                    self.scheduler.scoped(|s| {
                        s.schedule_job(EngineThreadCategory::GameObject, move || {
                            logic_component.run();
                        });
                    });
                }
            }

            thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
        }
    }
}
