use std::io::BufWriter;
use std::io::{self};
use std::num::NonZeroU32;
use std::time::Instant;

use engine::renderer::opengl_renderer::OpenGlRenderer;
use engine::renderer::Renderer;
use engine::scene::Scene;
use engine::Engine;
use engine::EngineThreadCategory;
use helper::translate_winit_keyboard_event;
use raw_window_handle::HasRawDisplayHandle;
use raw_window_handle::HasRawWindowHandle;
use util::job::Scheduler;
use util::logger::create_logger;
use util::logger::LogSeverity::Debug;
use util::logger::LogSeverity::Info;
use util::thread_pool_descriptor;
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event::KeyboardInput;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

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

    let engine = Engine::new(scheduler, logger_client, scene.clone());

    let event_loop = EventLoop::new();
    let mut previous_update = Instant::now();
    let inner_size = PhysicalSize::new(1024, 768);
    let window = WindowBuilder::new()
        .with_inner_size(inner_size)
        .build(&event_loop)
        .unwrap();

    #[cfg(target_os = "linux")]
    let renderer = OpenGlRenderer::new(
        window.raw_display_handle(),
        window.raw_window_handle(),
        NonZeroU32::new(inner_size.width).unwrap(),
        NonZeroU32::new(inner_size.height).unwrap(),
        Box::new(winit::platform::x11::register_xlib_error_hook),
    );

    #[cfg(target_os = "windows")]
    let renderer = OpenGlRenderer::new(
        window.raw_display_handle(),
        window.raw_window_handle(),
        NonZeroU32::new(inner_size.width).unwrap(),
        NonZeroU32::new(inner_size.height).unwrap(),
        (),
    );

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::MainEventsCleared => {
                let now = Instant::now();
                let delta_time = now - previous_update;
                previous_update = now;

                engine.update(delta_time);
                renderer.render(&scene);
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
                event: WindowEvent::Resized(size),
                ..
            } => {
                renderer.resize(size.width as usize, size.height as usize);
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
