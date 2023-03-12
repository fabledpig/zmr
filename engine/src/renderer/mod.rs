use crate::scene::Scene;

pub mod opengl_renderer;
mod opengl_shader;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub trait Renderer {
    fn render(&self, scene: &Scene);
    fn resize(&self, width: usize, height: usize);
}
