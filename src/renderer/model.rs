use std;
use gl;
use core::ffi::c_void;
use crate::renderer::Vertex;

pub struct Model {
    vao: gl::types::GLuint,
    vbo: (gl::types::GLuint, gl::types::GLsizei),
    ibo: (gl::types::GLuint, gl::types::GLsizei),
    render_mode: gl::types::GLenum,
}

#[derive(Copy, Clone, Debug)]
pub struct ModelInfo {
    pub vao: gl::types::GLuint,
    pub index_count: gl::types::GLsizei,
    pub render_mode: gl::types::GLenum,
}

impl Model {
    pub fn new(render_mode: gl::types::GLenum,
               verticies: &[Vertex],
               indices: &[gl::types::GLuint]) -> Model {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ibo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (verticies.len()
                            * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr,
                           std::mem::transmute(&verticies[0]),
                           gl::STATIC_DRAW);

            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (indices.len()
                            * std::mem::size_of::<gl::types::GLuint>()) as gl::types::GLsizeiptr,
                           std::mem::transmute(&indices[0]),
                           gl::STATIC_DRAW);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0,
                                    3,
                                    gl::FLOAT,
                                    gl::FALSE as gl::types::GLboolean,
                                    std::mem::size_of::<Vertex>() as gl::types::GLsizei,
                                    (std::ptr::null() as *const c_void)
                                        .offset(offset_of!(Vertex, position) as isize));

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1,
                                    3,
                                    gl::FLOAT,
                                    gl::FALSE as gl::types::GLboolean,
                                    std::mem::size_of::<Vertex>() as gl::types::GLsizei,
                                    (std::ptr::null() as *const c_void)
                                        .offset(offset_of!(Vertex, normal) as isize));

            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2,
                                    2,
                                    gl::FLOAT,
                                    gl::FALSE as gl::types::GLboolean,
                                    std::mem::size_of::<Vertex>() as gl::types::GLsizei,
                                    (std::ptr::null() as *const c_void)
                                        .offset(offset_of!(Vertex, uv) as isize));

            gl::BindVertexArray(0);
        }

        Model {
            vao,
            vbo: (vbo, verticies.len() as gl::types::GLsizei),
            ibo: (ibo, indices.len() as gl::types::GLsizei),
            render_mode,
        }
    }

    pub fn get_info(&self) -> ModelInfo {
        ModelInfo {
            vao: self.vao,
            index_count: self.ibo.1,
            render_mode: self.render_mode,
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.vbo.0);
            gl::DeleteBuffers(1, &mut self.ibo.0);
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}
