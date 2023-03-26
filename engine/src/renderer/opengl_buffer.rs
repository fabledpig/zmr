use super::gl;

pub struct OpenGlBuffer {
    buffer_id: gl::types::GLuint,
}

impl OpenGlBuffer {
    pub unsafe fn single() -> Self {
        let mut buffer_id = std::mem::zeroed();
        gl::GenBuffers(1, &mut buffer_id);

        Self { buffer_id }
    }

    #[allow(unused)]
    pub fn buffer_id(&self) -> gl::types::GLuint {
        self.buffer_id
    }

    pub fn bind(&self, target: gl::types::GLenum) {
        unsafe {
            gl::BindBuffer(target, self.buffer_id);
        }
    }
}

impl Drop for OpenGlBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer_id);
        }
    }
}
