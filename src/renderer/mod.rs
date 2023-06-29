pub mod model;
mod model_manager;
mod shader_manager;
mod texture_manager;

use self::model::ModelInfo;
use crate::light::Light;
use core::ffi::c_void;
use gamemath::Mat4;
use gamemath::Quat;
use gamemath::Vec2;
use gamemath::Vec3;
use gamemath::Vec4;
use gl;
use gl::types::{GLint, GLsizei, GLuint};
use std;
use std::collections::HashMap;
use std::mem::size_of;

static MAX_INSTANCES: usize = 10000;

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub uv: Vec2<f32>,
}

#[derive(Clone, Copy)]
pub struct RenderJob {
    pub model: ModelInfo,
    pub shader: GLuint,
    pub textures: usize,
    pub scale: Vec3<f32>,
    pub uv_size: Vec2<f32>,
    pub uv_offset: Vec2<f32>,
    pub position: Vec3<f32>,
    pub pivot: Vec3<f32>,
    pub rotation: Quat,
    pub tint: Vec4<f32>,
    pub emissive_tint: Vec4<f32>,
}

pub struct InstanceBuffer {
    model_matrix: Mat4,
    tint: Vec4<f32>,
    emissive_tint: Vec4<f32>,
    uv_size: Vec2<f32>,
    uv_offset: Vec2<f32>,
}

struct Camera {
    view: Mat4,
    projection: Mat4,
}

struct Skybox {
    shader: GLuint,
    model: ModelInfo,
    cube_map: GLuint,
}

pub struct Renderer<'a> {
    shader_manager: shader_manager::ShaderManager<'a>,
    model_manager: model_manager::ModelManager<'a>,
    texture_manager: texture_manager::TextureManager<'a>,
    render_target_framebuffer: Framebuffer,
    fullscreen_effect_framebuffer: Framebuffer,
    viewport: (Vec2<f32>, Vec2<f32>),
    job_vbo: GLuint,
    render_jobs: HashMap<GLuint, HashMap<GLuint, (ModelInfo, HashMap<usize, Vec<InstanceBuffer>>)>>,
    window_size: Vec2<f32>,
    skybox: Option<Skybox>,
    line_shader: GLuint,
    camera: Camera,
    light: Light,
}

struct Framebuffer {
    fbo: GLuint,
    size: (GLint, GLint),
    clear_color: Vec4<f32>,
    color_buffers: [(GLuint, GLuint); 2],
    depth_buffer: GLuint,
    current_front_buffer: usize,
    current_back_buffer: usize,
}

impl Framebuffer {
    fn new(width: GLint, height: GLint) -> Framebuffer {
        let mut fbo = 0;
        let color_buffers = [(0, 0), (0, 0)];

        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
        }

        let mut fb = Framebuffer {
            fbo,
            size: (0, 0),
            clear_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            color_buffers,
            depth_buffer: 0,
            current_front_buffer: 0,
            current_back_buffer: 1,
        };

        fb.resize(width, height);

        fb
    }

    fn get_front_buffer(&self) -> (GLuint, GLuint) {
        self.color_buffers[self.current_front_buffer]
    }

    fn get_back_buffer(&self) -> (GLuint, GLuint) {
        self.color_buffers[self.current_back_buffer]
    }

    fn activate(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        }
    }

    fn get_size(&self) -> Vec2<f32> {
        Vec2::new(self.size.0 as f32, self.size.1 as f32)
    }

    fn set_clear_color(&mut self, color: Vec4<f32>) {
        self.clear_color = color;
    }

    fn clear_buffers(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

            gl::ClearColor(
                self.clear_color.x,
                self.clear_color.y,
                self.clear_color.z,
                self.clear_color.w,
            );

            let attachments = [gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, attachments.as_ptr());
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);

            let attachments = [gl::COLOR_ATTACHMENT1];
            gl::DrawBuffers(1, attachments.as_ptr());
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            self.swap();

            gl::ClearColor(
                self.clear_color.x,
                self.clear_color.y,
                self.clear_color.z,
                self.clear_color.w,
            );

            let attachments = [gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, attachments.as_ptr());
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);

            let attachments = [gl::COLOR_ATTACHMENT1];
            gl::DrawBuffers(1, attachments.as_ptr());
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn swap(&mut self) {
        self.current_front_buffer ^= self.current_back_buffer;
        self.current_back_buffer ^= self.current_front_buffer;
        self.current_front_buffer ^= self.current_back_buffer;

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                self.color_buffers[self.current_front_buffer].0,
                0,
            );

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT1,
                gl::TEXTURE_2D,
                self.color_buffers[self.current_front_buffer].1,
                0,
            );
        }
    }

    fn resize(&mut self, width: GLint, height: GLint) {
        self.size = (width, height);
        self.current_front_buffer = 0;
        self.current_back_buffer = 1;

        let format = (gl::RGBA as GLint, gl::UNSIGNED_BYTE);

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

            gl::DeleteTextures(4, &mut self.color_buffers[0].0);
            gl::GenTextures(4, &mut self.color_buffers[0].0);

            gl::BindTexture(gl::TEXTURE_2D, self.color_buffers[0].0);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format.0,
                width,
                height,
                0,
                gl::RGBA as GLuint,
                format.1,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindTexture(gl::TEXTURE_2D, self.color_buffers[0].1);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format.0,
                width,
                height,
                0,
                gl::RGBA as GLuint,
                format.1,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindTexture(gl::TEXTURE_2D, self.color_buffers[1].0);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format.0,
                width,
                height,
                0,
                gl::RGBA as GLuint,
                format.1,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindTexture(gl::TEXTURE_2D, self.color_buffers[1].1);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format.0,
                width,
                height,
                0,
                gl::RGBA as GLuint,
                format.1,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                self.color_buffers[self.current_front_buffer].0,
                0,
            );

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT1,
                gl::TEXTURE_2D,
                self.color_buffers[self.current_front_buffer].1,
                0,
            );

            gl::DeleteRenderbuffers(1, &mut self.depth_buffer);
            gl::GenRenderbuffers(1, &mut self.depth_buffer);
            gl::BindRenderbuffer(gl::RENDERBUFFER, self.depth_buffer);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                self.depth_buffer,
            );

            let r = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);

            match r {
                gl::FRAMEBUFFER_COMPLETE => (),
                _ => panic!("Failed to resize framebuffer: {}", r),
            }
        }
    }
}

impl<'a> Renderer<'a> {
    pub fn new(
        window_size: Vec2<f32>,
        render_target_size: Vec2<f32>,
        shaders: &[(&'static str, &'static str, &'static str)],
    ) -> Renderer<'a> {
        let mut new_renderer = Renderer {
            shader_manager: shader_manager::ShaderManager::new(),
            model_manager: model_manager::ModelManager::new(),
            texture_manager: texture_manager::TextureManager::new(),
            render_target_framebuffer: Framebuffer::new(
                render_target_size.x as GLint,
                render_target_size.y as GLint,
            ),
            fullscreen_effect_framebuffer: Framebuffer::new(
                window_size.x as GLint,
                window_size.y as GLint,
            ),
            viewport: (window_size, Vec2::new(0.0, 0.0)),
            job_vbo: 0,
            render_jobs: HashMap::new(),
            window_size,
            skybox: None,
            line_shader: 0,
            camera: Camera {
                view: Mat4::identity(),
                projection: Mat4::perspective(45.0, window_size.x / window_size.y, 0.01, 1000.0),
            },
            light: Light {
                position: (2.5, 0.5, 2.5).into(),
                color: (1.0, 1.0, 1.0).into(),
            },
        };

        unsafe {
            gl::GenBuffers(1, &mut new_renderer.job_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, new_renderer.job_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (MAX_INSTANCES * size_of::<InstanceBuffer>()) as isize,
                std::ptr::null(),
                gl::STREAM_DRAW,
            );

            gl::VertexAttribDivisor(0, 0);
            gl::VertexAttribDivisor(1, 0);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        for shader in shaders.iter() {
            new_renderer.add_shader(shader.0, shader.1, shader.2);
        }

        new_renderer.line_shader = match new_renderer.shader_manager.get_shader("line") {
            Some(s) => s,
            None => 0,
        };

        new_renderer
    }

    pub fn set_clear_color(&mut self, color: Vec4<f32>) {
        self.render_target_framebuffer.set_clear_color(color);
    }

    pub fn rotate_camera(&mut self, rotation: Quat) {
        self.camera.view *= rotation.extract_matrix();
    }

    pub fn set_camera_position(&mut self, position: Vec3<f32>) {
        self.camera.view[3][0] = 0.0;
        self.camera.view[3][1] = 0.0;
        self.camera.view[3][2] = 0.0;
        self.camera.view.translate(position);
    }

    pub fn move_camera(&mut self, movement: Vec3<f32>) {
        self.camera.view.translate(movement);
    }

    pub fn set_view_matrix(&mut self, matrix: Mat4) {
        self.camera.view = matrix;
    }

    pub fn set_projection_matrix(&mut self, matrix: Mat4) {
        self.camera.projection = matrix;
    }

    pub fn get_viewport(&self) -> (Vec2<f32>, Vec2<f32>) {
        self.viewport
    }

    pub fn get_window_size(&self) -> Vec2<f32> {
        self.window_size
    }

    pub fn get_render_target_size(&self) -> Vec2<f32> {
        self.render_target_framebuffer.get_size()
    }

    pub fn rebuild_job_queues(&mut self) {
        for (_, shader_jobs) in self.render_jobs.iter_mut() {
            for (_, model_jobs) in shader_jobs.iter_mut() {
                for (_, texture_jobs) in model_jobs.1.iter_mut() {
                    texture_jobs.clear();
                }

                model_jobs.1.clear();
            }

            shader_jobs.clear();
        }

        let shaders = self.shader_manager.get_iterator();
        let models = self.model_manager.get_iterator();
        let texture_sets = self.texture_manager.get_texture_set_count();

        for shader in shaders {
            self.render_jobs.insert(shader.program, HashMap::new());
        }

        for model in models {
            let info = model.get_info();

            for shader in self.render_jobs.values_mut() {
                shader.insert(info.vao, (info, HashMap::new()));
            }
        }

        for set in 0..texture_sets {
            for shader_jobs in self.render_jobs.values_mut() {
                for model_jobs in shader_jobs.values_mut() {
                    model_jobs.1.insert(set, Vec::new());
                }
            }
        }
    }

    fn add_shader(&mut self, name: &'a str, vertex_src: &'static str, fragment_src: &'static str) {
        unsafe {
            self.shader_manager
                .create_program(name, vertex_src, fragment_src);
        }

        self.render_jobs.insert(
            self.shader_manager.get_shader(name).unwrap(),
            HashMap::new(),
        );
    }

    fn add_model(
        &mut self,
        name: &'a str,
        render_mode: gl::types::GLenum,
        verticies: &[Vertex],
        indices: &[gl::types::GLuint],
    ) {
        self.model_manager
            .add_model(name, render_mode, verticies, indices);

        let model_info = self.model_manager.get_model(name).unwrap().1;

        for shader in self.render_jobs.values_mut() {
            shader.insert(model_info.vao, (model_info, HashMap::new()));
        }
    }

    pub fn add_texture_set(&mut self, albedo: &'a str, emissive: &'a str) {
        let set = self.texture_manager.get_texture_set(albedo, emissive);

        for shader_jobs in self.render_jobs.values_mut() {
            for model_jobs in shader_jobs.values_mut() {
                model_jobs.1.insert(set.1, Vec::new());
            }
        }
    }

    pub fn add_cube_map(&mut self, name: &'a str, files: [&'a str; 6]) {
        self.texture_manager.load_cube_map(name, files);
    }

    pub fn get_shader(&self, name: &'a str) -> Option<GLuint> {
        self.shader_manager.get_shader(name)
    }

    pub fn get_texture_set(&mut self, albedo: &'a str, emissive: &'a str) -> usize {
        let result = self.texture_manager.get_texture_set(albedo, emissive);

        match result.0 {
            true => self.rebuild_job_queues(),
            false => (),
        }

        result.1
    }

    pub fn get_texture_set_sizes(&self, id: usize) -> (Vec2<f32>, Vec2<f32>) {
        self.texture_manager.get_texture_set_sizes(id)
    }

    pub fn get_model(&mut self, name: &'a str) -> Option<ModelInfo> {
        let result = self.model_manager.get_model(name);

        match result {
            Some(m) => {
                match m.0 {
                    true => self.rebuild_job_queues(),
                    false => (),
                }

                Some(m.1)
            }
            None => None,
        }
    }

    pub fn set_skybox(
        &mut self,
        shader_name: &'a str,
        model_name: &'a str,
        cube_map_name: &'a str,
    ) {
        let shader;
        let model;
        let cube_map;

        match self.shader_manager.get_shader(shader_name) {
            Some(s) => shader = s,
            None => {
                self.skybox = None;
                return;
            }
        };

        match self.model_manager.get_model(model_name) {
            Some(m) => model = m.1,
            None => {
                self.skybox = None;
                return;
            }
        }

        match self.texture_manager.get_cube_map(cube_map_name) {
            Some(cm) => cube_map = cm,
            None => {
                self.skybox = None;
                return;
            }
        }

        self.skybox = Some(Skybox {
            shader,
            model,
            cube_map,
        });
    }

    pub fn get_camera_position(&self) -> Vec3<f32> {
        let v = self.camera.view.inverted();

        Vec3::new(v[3][0], v[3][1], v[3][2])
    }

    pub unsafe fn clear_models(&mut self) {
        self.model_manager.clear_all_models();
    }

    pub unsafe fn clear_shaders(&mut self) {
        self.shader_manager.clear_all_shaders();
    }

    pub unsafe fn clear_textures(&mut self) {
        self.texture_manager.clear_all_textures();
    }

    pub unsafe fn clear_cube_maps(&mut self) {
        self.texture_manager.clear_all_cube_maps();
    }

    pub unsafe fn purge(&mut self) {
        self.shader_manager.clear_all_shaders();
        self.model_manager.clear_all_models();
        self.texture_manager.clear_all_textures();
        self.texture_manager.clear_all_cube_maps();
    }

    pub fn add_render_job(&mut self, job: RenderJob) {
        match self.render_jobs.get_mut(&job.shader) {
            Some(shader_jobs) => match shader_jobs.get_mut(&job.model.vao) {
                Some(model_jobs) => match model_jobs.1.get_mut(&job.textures) {
                    Some(texture_jobs) => {
                        let r = job.rotation.normalized().extract_matrix().transposed();
                        let mut p = Mat4::identity();
                        let mut s = Mat4::identity();
                        let mut t = Mat4::identity();

                        s.scale(job.scale);
                        p.translate(job.pivot);
                        t.translate(job.position);

                        let mut m = s;
                        m *= p;
                        m *= r;
                        m *= t;

                        texture_jobs.push(InstanceBuffer {
                            model_matrix: m,
                            tint: job.tint,
                            emissive_tint: job.emissive_tint,
                            uv_size: job.uv_size,
                            uv_offset: job.uv_offset,
                        });
                    }
                    None => (),
                },
                None => (),
            },
            None => (),
        }
    }

    pub fn resize(&mut self, new_size: Vec2<f32>) {
        let ratio = new_size.x / new_size.y;
        let render_target_size = self.render_target_framebuffer.get_size();
        let target_ratio = render_target_size.x / render_target_size.y;

        if ratio < target_ratio {
            self.viewport.0 = Vec2::new(new_size.x, (1.0 / target_ratio) * new_size.x);
            self.viewport.1 = Vec2::new(0.0, (new_size.y - self.viewport.0.y) / 2.0);
        } else if ratio > target_ratio {
            self.viewport.0 = Vec2::new(target_ratio * new_size.y, new_size.y);
            self.viewport.1 = Vec2::new((new_size.x - self.viewport.0.x) / 2.0, 0.0);
        } else {
            self.viewport.0 = new_size;
            self.viewport.1 = Vec2::new(0.0, 0.0);
        }

        self.viewport.0.x = (self.viewport.0.x as u32) as f32;
        self.viewport.0.y = (self.viewport.0.y as u32) as f32;
        self.viewport.1.x = (self.viewport.1.x as u32) as f32;
        self.viewport.1.y = (self.viewport.1.y as u32) as f32;
        self.window_size = new_size;

        unsafe {
            self.fullscreen_effect_framebuffer
                .resize(self.viewport.0.x as GLint, self.viewport.0.y as GLint);
        }
    }

    unsafe fn clear_all_buffers(&mut self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);

        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        self.fullscreen_effect_framebuffer.clear_buffers();
        self.render_target_framebuffer.clear_buffers();
    }

    pub unsafe fn present(&mut self) {
        let mut draw_call_count = 0;
        let render_target_size = self.render_target_framebuffer.get_size();

        self.clear_all_buffers();
        self.render_target_framebuffer.activate();
        gl::Viewport(
            0,
            0,
            render_target_size.x as GLsizei,
            render_target_size.y as GLsizei,
        );

        match self.skybox {
            Some(ref sb) => {
                gl::DepthMask(gl::FALSE);
                gl::CullFace(gl::FRONT);

                let mut view = self.camera.view;
                view[3][0] = 0.0;
                view[3][1] = 0.0;
                view[3][2] = 0.0;

                self.shader_manager.activate_shader(sb.shader);
                self.shader_manager.set_view_matrix(&view);
                self.shader_manager
                    .set_projection_matrix(&self.camera.projection);
                self.shader_manager.set_cube_map(sb.cube_map);
                self.model_manager.set_model(sb.model.vao);

                gl::DrawElements(
                    sb.model.render_mode,
                    sb.model.index_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );

                gl::CullFace(gl::BACK);
                gl::DepthMask(gl::TRUE);
            }
            None => (),
        }

        for (shader_id, shader_jobs) in self.render_jobs.iter_mut() {
            self.shader_manager.activate_shader(*shader_id);
            self.shader_manager.set_view_matrix(&self.camera.view);
            self.shader_manager
                .set_projection_matrix(&self.camera.projection);

            for (model_id, model_jobs) in shader_jobs.iter_mut() {
                self.model_manager.set_model(*model_id);
                let model_info = model_jobs.0;

                for (set_id, texture_jobs) in model_jobs.1.iter_mut() {
                    let mut remaining_job_count = texture_jobs.len();
                    let mut batches_done = 0;

                    if remaining_job_count > 0 {
                        let set = self.texture_manager.get_texture_set_data(*set_id);
                        self.shader_manager.set_albedo_texture(set.0);
                        self.shader_manager.set_emissive_texture(set.1);
                    }

                    while remaining_job_count > 0 {
                        let job_count = remaining_job_count.min(MAX_INSTANCES);
                        remaining_job_count -= job_count;

                        gl::BindBuffer(gl::ARRAY_BUFFER, self.job_vbo);
                        gl::BufferData(
                            gl::ARRAY_BUFFER,
                            (MAX_INSTANCES * size_of::<InstanceBuffer>()) as isize,
                            std::ptr::null(),
                            gl::STREAM_DRAW,
                        );

                        gl::BufferSubData(
                            gl::ARRAY_BUFFER,
                            0,
                            (job_count * size_of::<InstanceBuffer>()) as isize,
                            std::mem::transmute(&texture_jobs[batches_done * MAX_INSTANCES]),
                        );

                        gl::EnableVertexAttribArray(3);
                        gl::VertexAttribPointer(
                            3,
                            4,
                            gl::FLOAT,
                            gl::FALSE,
                            size_of::<InstanceBuffer>() as i32,
                            std::ptr::null(),
                        );

                        gl::EnableVertexAttribArray(4);
                        gl::VertexAttribPointer(
                            4,
                            4,
                            gl::FLOAT,
                            gl::FALSE,
                            size_of::<InstanceBuffer>() as i32,
                            (std::ptr::null() as *const c_void)
                                .offset((size_of::<Vec4<f32>>()) as isize),
                        );

                        gl::EnableVertexAttribArray(5);
                        gl::VertexAttribPointer(
                            5,
                            4,
                            gl::FLOAT,
                            gl::FALSE,
                            size_of::<InstanceBuffer>() as i32,
                            (std::ptr::null() as *const c_void)
                                .offset((size_of::<Vec4<f32>>() * 2) as isize),
                        );

                        gl::EnableVertexAttribArray(6);
                        gl::VertexAttribPointer(
                            6,
                            4,
                            gl::FLOAT,
                            gl::FALSE,
                            size_of::<InstanceBuffer>() as i32,
                            (std::ptr::null() as *const c_void)
                                .offset((size_of::<Vec4<f32>>() * 3) as isize),
                        );

                        gl::EnableVertexAttribArray(7);
                        gl::VertexAttribPointer(
                            7,
                            2,
                            gl::FLOAT,
                            gl::FALSE,
                            size_of::<InstanceBuffer>() as i32,
                            (std::ptr::null() as *const c_void).offset(offset_of!(
                                InstanceBuffer,
                                uv_size
                            )
                                as isize),
                        );

                        gl::EnableVertexAttribArray(8);
                        gl::VertexAttribPointer(
                            8,
                            2,
                            gl::FLOAT,
                            gl::FALSE,
                            size_of::<InstanceBuffer>() as i32,
                            (std::ptr::null() as *const c_void).offset(offset_of!(
                                InstanceBuffer,
                                uv_offset
                            )
                                as isize),
                        );

                        gl::EnableVertexAttribArray(9);
                        gl::VertexAttribPointer(
                            9,
                            4,
                            gl::FLOAT,
                            gl::FALSE,
                            size_of::<InstanceBuffer>() as i32,
                            (std::ptr::null() as *const c_void).offset(offset_of!(
                                InstanceBuffer,
                                tint
                            )
                                as isize),
                        );

                        gl::EnableVertexAttribArray(10);
                        gl::VertexAttribPointer(
                            10,
                            4,
                            gl::FLOAT,
                            gl::FALSE,
                            size_of::<InstanceBuffer>() as i32,
                            (std::ptr::null() as *const c_void).offset(offset_of!(
                                InstanceBuffer,
                                emissive_tint
                            )
                                as isize),
                        );

                        gl::VertexAttribDivisor(0, 0);
                        gl::VertexAttribDivisor(1, 0);
                        gl::VertexAttribDivisor(2, 0);
                        gl::VertexAttribDivisor(3, 1);
                        gl::VertexAttribDivisor(4, 1);
                        gl::VertexAttribDivisor(5, 1);
                        gl::VertexAttribDivisor(6, 1);
                        gl::VertexAttribDivisor(7, 1);
                        gl::VertexAttribDivisor(8, 1);
                        gl::VertexAttribDivisor(9, 1);
                        gl::VertexAttribDivisor(10, 1);

                        self.shader_manager.set_lights(&[self.light]);

                        let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
                        gl::DrawBuffers(2, attachments.as_ptr());
                        gl::DrawElementsInstanced(
                            model_info.render_mode,
                            model_info.index_count,
                            gl::UNSIGNED_INT,
                            std::ptr::null(),
                            job_count as i32,
                        );

                        batches_done += 1;
                        draw_call_count += 1;
                    }

                    texture_jobs.clear();
                }
            }
        }

        self.fullscreen_effect_framebuffer.activate();
        gl::Viewport(
            0,
            0,
            self.viewport.0.x as GLsizei,
            self.viewport.0.y as GLsizei,
        );

        let mut mat = Mat4::identity();
        let s = self.shader_manager.get_shader("copy").unwrap();
        let m = self.model_manager.get_model("sprite").unwrap();

        self.shader_manager.activate_shader(s);
        self.shader_manager.set_view_matrix(&mat);
        self.shader_manager.set_projection_matrix(&mat);
        self.shader_manager
            .set_albedo_texture(self.render_target_framebuffer.get_front_buffer().0);
        self.shader_manager
            .set_emissive_texture(self.render_target_framebuffer.get_front_buffer().1);
        self.model_manager.set_model(m.1.vao);

        mat.translate(Vec3::new(-1.0, 1.0, 0.0));

        let data = InstanceBuffer {
            model_matrix: mat,
            tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
            emissive_tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
            uv_size: Vec2::new(1.0, -1.0),
            uv_offset: Vec2::new(0.0, 0.0),
        };

        gl::BindBuffer(gl::ARRAY_BUFFER, self.job_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (MAX_INSTANCES * size_of::<InstanceBuffer>()) as isize,
            std::ptr::null(),
            gl::STREAM_DRAW,
        );

        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            0,
            size_of::<InstanceBuffer>() as isize,
            std::mem::transmute(&data),
        );

        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(
            3,
            4,
            gl::FLOAT,
            gl::FALSE,
            size_of::<InstanceBuffer>() as i32,
            std::ptr::null(),
        );

        gl::EnableVertexAttribArray(4);
        gl::VertexAttribPointer(
            4,
            4,
            gl::FLOAT,
            gl::FALSE,
            size_of::<InstanceBuffer>() as i32,
            (std::ptr::null() as *const c_void).offset((size_of::<Vec4<f32>>()) as isize),
        );

        gl::EnableVertexAttribArray(5);
        gl::VertexAttribPointer(
            5,
            4,
            gl::FLOAT,
            gl::FALSE,
            size_of::<InstanceBuffer>() as i32,
            (std::ptr::null() as *const c_void).offset((size_of::<Vec4<f32>>() * 2) as isize),
        );

        gl::EnableVertexAttribArray(6);
        gl::VertexAttribPointer(
            6,
            4,
            gl::FLOAT,
            gl::FALSE,
            size_of::<InstanceBuffer>() as i32,
            (std::ptr::null() as *const c_void).offset((size_of::<Vec4<f32>>() * 3) as isize),
        );

        gl::EnableVertexAttribArray(7);
        gl::VertexAttribPointer(
            7,
            2,
            gl::FLOAT,
            gl::FALSE,
            size_of::<InstanceBuffer>() as i32,
            (std::ptr::null() as *const c_void)
                .offset(offset_of!(InstanceBuffer, uv_size) as isize),
        );

        gl::EnableVertexAttribArray(8);
        gl::VertexAttribPointer(
            8,
            2,
            gl::FLOAT,
            gl::FALSE,
            size_of::<InstanceBuffer>() as i32,
            (std::ptr::null() as *const c_void)
                .offset(offset_of!(InstanceBuffer, uv_offset) as isize),
        );

        gl::EnableVertexAttribArray(9);
        gl::VertexAttribPointer(
            9,
            4,
            gl::FLOAT,
            gl::FALSE,
            size_of::<InstanceBuffer>() as i32,
            (std::ptr::null() as *const c_void).offset(offset_of!(InstanceBuffer, tint) as isize),
        );

        gl::EnableVertexAttribArray(10);
        gl::VertexAttribPointer(
            10,
            4,
            gl::FLOAT,
            gl::FALSE,
            size_of::<InstanceBuffer>() as i32,
            (std::ptr::null() as *const c_void)
                .offset(offset_of!(InstanceBuffer, emissive_tint) as isize),
        );

        gl::VertexAttribDivisor(0, 0);
        gl::VertexAttribDivisor(1, 0);
        gl::VertexAttribDivisor(2, 0);
        gl::VertexAttribDivisor(3, 1);
        gl::VertexAttribDivisor(4, 1);
        gl::VertexAttribDivisor(5, 1);
        gl::VertexAttribDivisor(6, 1);
        gl::VertexAttribDivisor(7, 1);
        gl::VertexAttribDivisor(8, 1);
        gl::VertexAttribDivisor(9, 1);
        gl::VertexAttribDivisor(10, 1);

        let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
        gl::DrawBuffers(2, attachments.as_ptr());
        gl::DrawElementsInstanced(
            m.1.render_mode,
            m.1.index_count,
            gl::UNSIGNED_INT,
            std::ptr::null(),
            1,
        );

        let mut mat = Mat4::identity();

        self.fullscreen_effect_framebuffer.swap();
        let effect_back_buffer = self.fullscreen_effect_framebuffer.get_back_buffer();
        let s = self.shader_manager.get_shader("vertical_blur").unwrap();

        self.shader_manager.activate_shader(s);
        self.shader_manager.set_view_matrix(&mat);
        self.shader_manager.set_projection_matrix(&mat);
        self.shader_manager
            .set_emissive_texture(effect_back_buffer.1);

        let attachments = [gl::COLOR_ATTACHMENT1];
        gl::DrawBuffers(1, attachments.as_ptr());
        gl::DrawElementsInstanced(
            m.1.render_mode,
            m.1.index_count,
            gl::UNSIGNED_INT,
            std::ptr::null(),
            1,
        );

        self.fullscreen_effect_framebuffer.swap();
        let effect_back_buffer = self.fullscreen_effect_framebuffer.get_back_buffer();
        let s = self.shader_manager.get_shader("horizontal_blur").unwrap();

        self.shader_manager.activate_shader(s);
        self.shader_manager.set_view_matrix(&mat);
        self.shader_manager.set_projection_matrix(&mat);
        self.shader_manager
            .set_emissive_texture(effect_back_buffer.1);

        let attachments = [gl::COLOR_ATTACHMENT1];
        gl::DrawBuffers(1, attachments.as_ptr());
        gl::DrawElementsInstanced(
            m.1.render_mode,
            m.1.index_count,
            gl::UNSIGNED_INT,
            std::ptr::null(),
            1,
        );

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::Viewport(
            self.viewport.1.x as GLint,
            self.viewport.1.y as GLint,
            self.viewport.0.x as GLsizei,
            self.viewport.0.y as GLsizei,
        );

        let s = self.shader_manager.get_shader("add_emissive").unwrap();

        self.shader_manager.activate_shader(s);
        self.shader_manager.set_view_matrix(&mat);
        self.shader_manager.set_projection_matrix(&mat);
        self.shader_manager
            .set_albedo_texture(self.fullscreen_effect_framebuffer.get_front_buffer().0);
        self.shader_manager
            .set_emissive_texture(self.fullscreen_effect_framebuffer.get_front_buffer().1);
        self.model_manager.set_model(m.1.vao);

        mat.translate(Vec3::new(-1.0, 1.0, 0.0));

        let data = InstanceBuffer {
            model_matrix: mat,
            tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
            emissive_tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
            uv_size: Vec2::new(1.0, -1.0),
            uv_offset: Vec2::new(0.0, 0.0),
        };

        gl::BindBuffer(gl::ARRAY_BUFFER, self.job_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (MAX_INSTANCES * size_of::<InstanceBuffer>()) as isize,
            std::ptr::null(),
            gl::STREAM_DRAW,
        );

        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            0,
            size_of::<InstanceBuffer>() as isize,
            std::mem::transmute(&data),
        );

        gl::DrawElementsInstanced(
            m.1.render_mode,
            m.1.index_count,
            gl::UNSIGNED_INT,
            std::ptr::null(),
            1,
        );
    }
}
