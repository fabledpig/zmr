use std::ffi::c_void;
use std::sync::Arc;

use super::gl;
use super::opengl_buffer::OpenGlBuffer;

pub struct OpenGlVertexArray {
    vertex_array_id: gl::types::GLuint,
    vertex_buffer_objects: Vec<Arc<OpenGlBuffer>>,
}

impl OpenGlVertexArray {
    #[allow(unused)]
    pub fn vertex_array_id(&self) -> gl::types::GLuint {
        self.vertex_array_id
    }

    #[allow(unused)]
    pub fn vertex_buffer_objects(&self) -> &[Arc<OpenGlBuffer>] {
        self.vertex_buffer_objects.as_ref()
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vertex_array_id);
        }
    }
}

impl Drop for OpenGlVertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array_id);
        }
    }
}

pub struct OpenGlVertexAttribPointer {
    index: gl::types::GLuint,
    size: gl::types::GLint,
    type_: gl::types::GLenum,
    normalized: gl::types::GLboolean,
    stride: gl::types::GLsizei,
    pointer: *const c_void,
}

impl OpenGlVertexAttribPointer {
    pub fn new(
        index: gl::types::GLuint,
        size: gl::types::GLint,
        type_: gl::types::GLenum,
        normalized: gl::types::GLboolean,
        stride: gl::types::GLsizei,
        pointer: *const c_void,
    ) -> Self {
        Self {
            index,
            size,
            type_,
            normalized,
            stride,
            pointer,
        }
    }
}

pub struct OpenGlVertexArrayBuilder {
    vertex_buffer_objects: Vec<Arc<OpenGlBuffer>>,
    attrib_pointers: Vec<Vec<OpenGlVertexAttribPointer>>,
}

impl OpenGlVertexArrayBuilder {
    pub fn new() -> Self {
        Self {
            vertex_buffer_objects: Vec::new(),
            attrib_pointers: Vec::new(),
        }
    }

    pub fn add_buffer(
        &mut self,
        vertex_buffer_object: Arc<OpenGlBuffer>,
        attrib_pointers: Vec<OpenGlVertexAttribPointer>,
    ) -> &mut Self {
        self.vertex_buffer_objects.push(vertex_buffer_object);
        self.attrib_pointers.push(attrib_pointers);
        self
    }

    pub unsafe fn build(&mut self) -> OpenGlVertexArray {
        let mut vertex_array_id = std::mem::zeroed();
        gl::GenVertexArrays(1, &mut vertex_array_id);
        gl::BindVertexArray(vertex_array_id);

        self.vertex_buffer_objects
            .iter()
            .zip(self.attrib_pointers.iter())
            .for_each(|(buffer, attrib_pointers)| {
                buffer.bind(gl::ARRAY_BUFFER);

                for attrib_pointer in attrib_pointers {
                    gl::VertexAttribPointer(
                        attrib_pointer.index,
                        attrib_pointer.size,
                        attrib_pointer.type_,
                        attrib_pointer.normalized,
                        attrib_pointer.stride,
                        attrib_pointer.pointer,
                    );

                    gl::EnableVertexAttribArray(attrib_pointer.index);
                }
            });

        let vertex_buffer_objects = self.vertex_buffer_objects.clone();
        OpenGlVertexArray {
            vertex_array_id,
            vertex_buffer_objects,
        }
    }
}

impl Default for OpenGlVertexArrayBuilder {
    fn default() -> Self {
        Self::new()
    }
}
