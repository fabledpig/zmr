use std::io::{self, BufWriter};

use engine::{scene::Scene, Engine, EngineThreadCategory};
use util::{
    job::Scheduler,
    logger::{
        create_logger,
        LogSeverity::{Debug, Info},
    },
    thread_pool_descriptor,
};

thread_pool_descriptor!(EngineThreadCategory, Logger: 1, GameObject: 4);

fn main() {
    let scheduler = Scheduler::new(ThreadPoolDescriptor {});
    let (logger_server, logger_client) = create_logger(64, Box::new(BufWriter::new(io::stdout())));
    scheduler.schedule_job(EngineThreadCategory::Logger, || {
        logger_server.work();
    });
    logger_client.log(Info, "Game started");

    let scene = Scene::new();

    let game_object = scene.add_game_object();
    game_object.add_logic_component(move |engine_context, _| {
        engine_context
            .logger_client()
            .log(Debug, "Hello from logic component!");

        engine_context.stop_engine();
    });

    let engine = Engine::new(scheduler, logger_client);
    engine.work(&scene);
}
