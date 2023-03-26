use std::collections::HashMap;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::ptr;

use glutin::config::Config;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::ContextAttributesBuilder;
use glutin::context::NotCurrentContext;
use glutin::context::PossiblyCurrentContext;
use glutin::display::Display;
use glutin::display::DisplayApiPreference;
use glutin::prelude::GlDisplay;
use glutin::prelude::NotCurrentGlContextSurfaceAccessor;
use glutin::surface::GlSurface;
use glutin::surface::Surface;
use glutin::surface::SurfaceAttributesBuilder;
use glutin::surface::WindowSurface;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;

use super::gl;
use super::opengl_buffer::OpenGlBuffer;
use super::opengl_shader::OpenGlShader;
use super::opengl_shader::OpenGlShaderProgram;
use super::Renderer;
use super::ShaderId;
use crate::scene::Scene;

#[cfg(target_os = "linux")]
pub type XlibErrorHookRegistrar = glutin::api::glx::XlibErrorHookRegistrar;

#[cfg(target_os = "windows")]
pub type XlibErrorHookRegistrar = ();

pub struct OpenGlRenderer {
    shader_programs: HashMap<ShaderId, OpenGlShaderProgram>,
    vao: gl::types::GLuint,
    vbo: OpenGlBuffer,
    gl_surface: Surface<WindowSurface>,
    gl_context: PossiblyCurrentContext,
}

impl OpenGlRenderer {
    pub fn new(
        raw_display_handle: RawDisplayHandle,
        raw_window_handle: RawWindowHandle,
        width: NonZeroU32,
        height: NonZeroU32,
        xlib_error_hook_registrar: XlibErrorHookRegistrar,
    ) -> Self {
        unsafe {
            let gl_display = Self::create_display(
                raw_display_handle,
                raw_window_handle,
                xlib_error_hook_registrar,
            );
            let gl_config = Self::create_config(&gl_display);
            let mut not_current_gl_context = Some(Self::create_not_current_context(
                raw_window_handle,
                &gl_display,
                &gl_config,
            ));
            let gl_surface =
                Self::create_surface(&gl_display, &gl_config, raw_window_handle, width, height);
            let gl_context =
                Self::make_context_current(not_current_gl_context.take().unwrap(), &gl_surface);

            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            let vertex_shader = OpenGlShader::new(gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = OpenGlShader::new(gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);
            let shader_program = OpenGlShaderProgram::new(&[vertex_shader, fragment_shader]);

            let mut shader_programs = HashMap::new();
            shader_programs.insert(ShaderId::BuiltIn, shader_program);

            let mut vao = std::mem::zeroed();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let vbo = OpenGlBuffer::single();
            vbo.bind(gl::ARRAY_BUFFER);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl::EnableVertexAttribArray(1);

            Self {
                shader_programs,
                vao,
                vbo,
                gl_surface,
                gl_context,
            }
        }
    }

    unsafe fn create_display(
        raw_display_handle: RawDisplayHandle,
        #[allow(unused)] raw_window_handle: RawWindowHandle,
        #[allow(unused)] xlib_error_hook_registrar: XlibErrorHookRegistrar,
    ) -> Display {
        #[cfg(target_os = "windows")]
        let preference = DisplayApiPreference::Wgl(Some(raw_window_handle));

        #[cfg(target_os = "linux")]
        let preference = DisplayApiPreference::Glx(xlib_error_hook_registrar);

        Display::new(raw_display_handle, preference).unwrap()
    }

    unsafe fn create_config(gl_display: &Display) -> Config {
        gl_display
            .find_configs(ConfigTemplateBuilder::new().build())
            .unwrap()
            .next()
            .unwrap()
    }

    unsafe fn create_not_current_context(
        raw_window_handle: RawWindowHandle,
        gl_display: &Display,
        gl_config: &Config,
    ) -> NotCurrentContext {
        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
        gl_display
            .create_context(gl_config, &context_attributes)
            .unwrap()
    }

    unsafe fn create_surface(
        gl_display: &Display,
        gl_config: &Config,
        raw_window_handle: RawWindowHandle,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Surface<WindowSurface> {
        gl_display
            .create_window_surface(
                gl_config,
                &SurfaceAttributesBuilder::<WindowSurface>::new().build(
                    raw_window_handle,
                    width,
                    height,
                ),
            )
            .unwrap()
    }

    unsafe fn make_context_current(
        not_current_gl_context: NotCurrentContext,
        gl_surface: &Surface<WindowSurface>,
    ) -> PossiblyCurrentContext {
        not_current_gl_context.make_current(gl_surface).unwrap()
    }
}

impl Renderer for OpenGlRenderer {
    fn render(&self, _scene: &Scene) {
        unsafe {
            self.shader_programs
                .get(&ShaderId::BuiltIn)
                .unwrap()
                .use_program();

            gl::BindVertexArray(self.vao);
            self.vbo.bind(gl::ARRAY_BUFFER);

            gl::ClearColor(0.5, 0.5, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
    }

    fn resize(&self, width: usize, height: usize) {
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }
}

impl Drop for OpenGlRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vao);
        }
    }
}

#[rustfmt::skip]
static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
     0.0,  0.5,  0.0,  1.0,  0.0,
     0.5, -0.5,  0.0,  0.0,  1.0,
];

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 330

layout (location = 0) in vec2 pos;
layout (location = 1) in vec3 color;

out vec3 vertex_color;

void main() {
    gl_Position = vec4(pos.x, pos.y, 0.0, 1.0);
    vertex_color = color;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 330

in vec3 vertex_color;

out vec4 fragment_color;

void main() {
    fragment_color = vec4(vertex_color, 1.0);
}
\0";
