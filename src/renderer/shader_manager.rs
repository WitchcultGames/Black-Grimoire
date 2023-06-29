use std;
use gl;
use gamemath::Mat4;
use crate::light::Light;
use std::collections::hash_map::{HashMap, Values};

pub struct ShaderData {
    pub program: gl::types::GLuint,
    albedo_location: gl::types::GLint,
    emissive_location: gl::types::GLint,
}

pub struct ShaderManager<'a> {
    programs: std::collections::HashMap<&'a str, ShaderData>,
    current_program: ShaderData,
}

impl<'a> ShaderManager<'a> {
    pub fn new() -> ShaderManager<'a> {
        ShaderManager {
            programs: HashMap::new(),
            current_program: ShaderData {
                program: 0,
                albedo_location: 0,
                emissive_location: 0,
            },
        }
    }

    pub fn get_iterator(&self) -> Values<&'a str, ShaderData> {
        self.programs.values()
    }

    pub fn get_shader(&self, name: &str) -> Option<gl::types::GLuint> {
        match self.programs.get(name) {
            Some(shader) => Some(shader.program),
            None => None,
        }
    }

    pub unsafe fn activate_shader(&mut self, shader: gl::types::GLuint) {
        if self.current_program.program != shader {
            gl::UseProgram(shader);

            for s in self.programs.values() {
                if s.program == shader {
                    self.current_program = ShaderData {
                        program: s.program,
                        albedo_location: s.albedo_location,
                        emissive_location: s.emissive_location,
                    };
                    break;
                }
            }
        }
    }

    pub unsafe fn set_albedo_texture(&mut self, texture: gl::types::GLuint) {
        if self.current_program.albedo_location >= 0 {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::Uniform1i(self.current_program.albedo_location, 0);
        } else {
            let mut key = "";

            for (name, program) in self.programs.iter() {
                if self.current_program.program == program.program {
                    key = name;
                    break;
                }
            }

            println!("Could not set albedo texture, shader \'{}\' does not have the correct sampler!", key);
        }
    }

    pub unsafe fn set_emissive_texture(&mut self, texture: gl::types::GLuint) {
        if self.current_program.emissive_location >= 0 {
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::Uniform1i(self.current_program.emissive_location, 1);
        } else {
            let mut key = "";

            for (name, program) in self.programs.iter() {
                if self.current_program.program == program.program {
                    key = name;
                    break;
                }
            }

            println!("Could not set emissive texture! shader \'{}\' does not have the correct sampler!", key);
        }
    }

    pub unsafe fn set_cube_map(&mut self, texture: gl::types::GLuint) {
        let loc = gl::GetUniformLocation(self.current_program.program,
                                         std::ffi::CString::new("cube_map")
                                             .unwrap()
                                             .as_ptr());

        gl::ActiveTexture(gl::TEXTURE0 + 0);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);
        gl::Uniform1i(loc, 0);
    }

    pub unsafe fn set_view_matrix(&mut self, matrix: &Mat4) {
        let loc = gl::GetUniformLocation(self.current_program.program,
                                         std::ffi::CString::new("view_matrix")
                                             .unwrap()
                                             .as_ptr());

        gl::UniformMatrix4fv(loc, 1, gl::FALSE, std::mem::transmute(matrix));
    }

    pub unsafe fn set_projection_matrix(&mut self, matrix: &Mat4) {
        let loc = gl::GetUniformLocation(self.current_program.program,
                                         std::ffi::CString::new("projection_matrix")
                                             .unwrap()
                                             .as_ptr());

        gl::UniformMatrix4fv(loc, 1, gl::FALSE, std::mem::transmute(matrix));
    }

    pub unsafe fn set_lights(&mut self, lights: &[Light]) {
        let count_loc = gl::GetUniformLocation(self.current_program.program,
                                               std::ffi::CString::new("light_count")
                                                   .unwrap()
                                                   .as_ptr());

        let light_loc = gl::GetUniformLocation(self.current_program.program,
                                               std::ffi::CString::new("lights")
                                                   .unwrap()
                                                   .as_ptr());

        let count = lights.len().min(8) as i32;

        gl::Uniform1i(count_loc, count);
        gl::Uniform3fv(light_loc, count * 2, std::mem::transmute(&lights[0]));
    }

    pub unsafe fn create_program(&mut self,
                      name: &'a str,
                      vertex_src: &'static str,
                      fragment_src: &'static str) {
        let vs = self.compile_glsl(gl::VERTEX_SHADER, vertex_src);
        let fs = self.compile_glsl(gl::FRAGMENT_SHADER, fragment_src);
        let p = self.link_program(vs, fs);

        let albedo_location = gl::GetUniformLocation(p,
                                                     std::ffi::CString::new("albedo_texture")
                                                         .unwrap()
                                                         .as_ptr());

        let emissive_location = gl::GetUniformLocation(p,
                                                       std::ffi::CString::new("emissive_texture")
                                                           .unwrap()
                                                           .as_ptr());

        self.programs.insert(name, ShaderData {
            program: p,
            albedo_location,
            emissive_location,
        });
    }

    pub unsafe fn clear_all_shaders(&mut self) {
        for (_, shader) in self.programs.iter() {
            gl::DeleteProgram(shader.program);
        }

        self.programs.clear();
    }

    fn compile_glsl(&self, shader_type: gl::types::GLenum, src: &str) -> gl::types::GLuint {
        let shader;

        unsafe {
            shader = gl::CreateShader(shader_type);

            gl::ShaderSource(shader,
                             1,
                             &(std::ffi::CString::new(src.as_bytes()).unwrap()).as_ptr(),
                             std::ptr::null());

            gl::CompileShader(shader);

            let mut status = gl::FALSE as gl::types::GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            if status != (gl::TRUE as gl::types::GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = std::vec::Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);

                gl::GetShaderInfoLog(shader,
                                     len,
                                     std::ptr::null_mut(),
                                     buf.as_mut_ptr() as *mut gl::types::GLchar);

                panic!("{}",
                       std::str::from_utf8(&buf)
                           .ok()
                           .expect("ShaderInfoLog not valid utf8!"));
            }
        }

        shader
    }

    fn link_program(&self,
                    vertex_shader: gl::types::GLuint,
                    fragment_shader: gl::types::GLuint) -> gl::types::GLuint {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            let mut status = gl::FALSE as gl::types::GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            if status != (gl::TRUE as gl::types::GLint) {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = std::vec::Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);

                gl::GetProgramInfoLog(program,
                                      len,
                                      std::ptr::null_mut(),
                                      buf.as_mut_ptr() as *mut gl::types::GLchar);

                panic!("{}",
                       std::str::from_utf8(&buf)
                           .ok()
                           .expect("ProgramInfoLog not valid utf8!"));
            }

            program
        }
    }
}

impl<'a> Drop for ShaderManager<'a> {
    fn drop(&mut self) {
        unsafe { self.clear_all_shaders(); };
    }
}
