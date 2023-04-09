use super::gl;

pub struct OpenGlBuffer {
    buffer_id: gl::types::GLuint,
}

impl OpenGlBuffer {
    pub unsafe fn single() -> Self {
        let mut buffer_id = std::mem::zeroed();
        gl::CreateBuffers(1, &mut buffer_id);

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

    pub fn buffer_data<T>(&self, data: &[T], usage: gl::types::GLenum) {
        unsafe {
            gl::NamedBufferData(
                self.buffer_id,
                (data.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr,
                data.as_ptr() as *const _,
                usage,
            );
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
