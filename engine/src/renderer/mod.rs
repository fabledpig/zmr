use crate::scene::Scene;

mod opengl_buffer;
pub mod opengl_renderer;
mod opengl_shader;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub trait Renderer {
    fn render(&self, scene: &Scene);
    fn resize(&self, width: usize, height: usize);
}

#[derive(PartialEq, Eq, Hash)]
pub enum ShaderId {
    BuiltIn,
    Custom(String),
}
