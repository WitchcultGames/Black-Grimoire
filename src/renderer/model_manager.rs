use crate::renderer::model::{Model, ModelInfo};
use crate::renderer::Vertex;
use crate::utilities::read_struct;
use gl;
use std;
use std::collections::hash_map::{HashMap, Values};
use std::fs::File;
use std::path::Path;

pub struct ModelManager<'a> {
    models: std::collections::HashMap<&'a str, Model>,
}

impl<'a> ModelManager<'a> {
    pub fn new() -> ModelManager<'a> {
        ModelManager {
            models: HashMap::new(),
        }
    }

    pub fn get_iterator(&self) -> Values<&'a str, Model> {
        self.models.values()
    }

    pub fn add_model(
        &mut self,
        name: &'a str,
        render_mode: gl::types::GLenum,
        verticies: &[Vertex],
        indices: &[gl::types::GLuint],
    ) {
        self.models
            .insert(name, Model::new(render_mode, verticies, indices));
    }

    pub unsafe fn load_model(&mut self, name: &'a str) {
        let path_string = format!("res/models/{}", name);
        let path = Path::new(path_string.as_str());
        let mut file = File::open(path).unwrap();

        let vertex_count: u32 = match read_struct(&mut file) {
            Ok(v) => v,
            Err(e) => panic!("Failed to load model '{}': {}", name, e),
        };

        let index_count: u32 = match read_struct(&mut file) {
            Ok(v) => v,
            Err(e) => panic!("Failed to load model '{}': {}", name, e),
        };

        let mut verticies = Vec::new();
        let mut indices = Vec::new();

        for _i in 0..vertex_count {
            let vertex: Vertex = match read_struct(&mut file) {
                Ok(v) => v,
                Err(e) => panic!("Failed to load model '{}': {}", name, e),
            };

            verticies.push(vertex);
        }

        for _ in 0..index_count {
            let index: u32 = match read_struct(&mut file) {
                Ok(v) => v,
                Err(e) => panic!("Failed to load model '{}': {}", name, e),
            };

            indices.push(index);
        }

        self.add_model(name, gl::TRIANGLES, &verticies, &indices);
    }

    pub fn clear_all_models(&mut self) {
        self.models.clear();
    }

    pub fn get_model(&mut self, name: &'a str) -> Option<(bool, ModelInfo)> {
        let mut model;

        match self.models.get(name) {
            Some(m) => model = Some((false, m.get_info())),
            None => model = None,
        }

        match model {
            Some(_m) => (),
            None => {
                unsafe {
                    self.load_model(name);
                };

                match self.models.get(name) {
                    Some(m) => model = Some((true, m.get_info())),
                    None => model = None,
                }
            }
        }

        model
    }

    pub unsafe fn set_model(&self, model_vao: gl::types::GLuint) {
        gl::BindVertexArray(model_vao);
    }
}

impl<'a> Drop for ModelManager<'a> {
    fn drop(&mut self) {
        self.clear_all_models();
    }
}
