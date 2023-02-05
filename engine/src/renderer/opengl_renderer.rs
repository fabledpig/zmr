use std::{ffi::CString, ptr};

use glutin::prelude::GlDisplay;

use crate::scene::Scene;

use super::Renderer;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub struct OpenGlRenderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
}

impl OpenGlRenderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
        unsafe {
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            let vertex_shader = Self::create_shader(gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = Self::create_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);
            gl::UseProgram(program);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            let mut vao = std::mem::zeroed();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

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

            Self { program, vao, vbo }
        }
    }

    unsafe fn create_shader(shader_type: gl::types::GLenum, source: &[u8]) -> gl::types::GLuint {
        let shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, [source.as_ptr().cast()].as_ptr(), ptr::null());
        gl::CompileShader(shader);
        shader
    }
}

impl Renderer for OpenGlRenderer {
    fn render(&self, _scene: &Scene) {
        unsafe {
            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::ClearColor(0.5, 0.5, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
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
