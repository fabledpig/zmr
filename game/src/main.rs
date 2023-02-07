use std::io::BufWriter;
use std::io::{self};
use std::time::Instant;

use engine::scene::Scene;
use engine::Engine;
use engine::EngineThreadCategory;
use helper::translate_winit_keyboard_event;
use util::job::Scheduler;
use util::logger::create_logger;
use util::logger::LogSeverity::Debug;
use util::logger::LogSeverity::Info;
use util::thread_pool_descriptor;
use winit::event::Event;
use winit::event::KeyboardInput;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::window::Window;

mod helper;

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
    });

    let engine = Engine::new(scheduler, logger_client, scene);

    let event_loop = EventLoop::new();
    let _window = Window::new(&event_loop).unwrap();

    let mut previous_update = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::MainEventsCleared => {
                let now = Instant::now();
                let delta_time = now - previous_update;
                previous_update = now;

                engine.update(delta_time);
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(virtual_key_code),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                translate_winit_keyboard_event(
                    engine.engine_context().input_handler(),
                    virtual_key_code,
                    state,
                );
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            }
            _ => (),
        }
    });
}
