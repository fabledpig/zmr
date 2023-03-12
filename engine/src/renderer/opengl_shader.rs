use std::ptr;

use super::gl;

pub struct OpenGlShaderProgram {
    shader_program_id: gl::types::GLuint,
}

impl OpenGlShaderProgram {
    pub unsafe fn new<T>(shaders: &T) -> Self
    where
        for<'a> &'a T: IntoIterator<Item = &'a OpenGlShader>,
    {
        let program = gl::CreateProgram();
        for shader in shaders {
            gl::AttachShader(program, shader.shader_id());
        }
        gl::LinkProgram(program);

        Self {
            shader_program_id: program,
        }
    }

    #[allow(unused)]
    pub fn shader_program_id(&self) -> gl::types::GLuint {
        self.shader_program_id
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.shader_program_id);
        }
    }
}

impl Drop for OpenGlShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program_id);
        }
    }
}

pub struct OpenGlShader {
    shader_id: gl::types::GLuint,
}

impl OpenGlShader {
    pub unsafe fn new(shader_type: gl::types::GLenum, source: &[u8]) -> Self {
        let shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, [source.as_ptr().cast()].as_ptr(), ptr::null());
        gl::CompileShader(shader);

        Self { shader_id: shader }
    }

    pub fn shader_id(&self) -> u32 {
        self.shader_id
    }
}

impl Drop for OpenGlShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.shader_id);
        }
    }
}
