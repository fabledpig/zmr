use crate::scene::Scene;

pub mod opengl_renderer;

pub trait Renderer {
    fn render(&self, scene: &Scene);
    fn resize(&self, width: usize, height: usize);
}
