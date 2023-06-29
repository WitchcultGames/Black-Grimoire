extern crate libc;
extern crate lodepng;
extern crate rgb;

use self::rgb::*;
use gamemath::Vec2;
use gl;
use std;
use std::collections::hash_map::HashMap;

pub struct TextureManager<'a> {
    textures: std::collections::HashMap<&'a str, (gl::types::GLuint, Vec2<f32>)>,
    cube_maps: std::collections::HashMap<&'a str, gl::types::GLuint>,
    texture_sets: Vec<(gl::types::GLuint, gl::types::GLuint)>,
}

impl<'a> TextureManager<'a> {
    pub fn new() -> TextureManager<'a> {
        TextureManager {
            textures: HashMap::new(),
            cube_maps: HashMap::new(),
            texture_sets: Vec::new(),
        }
    }

    pub fn get_texture_set_count(&self) -> usize {
        self.texture_sets.len()
    }

    pub fn get_texture_set_sizes(&self, id: usize) -> (Vec2<f32>, Vec2<f32>) {
        if id < self.texture_sets.len() {
            let set = self.texture_sets[id];
            let mut result = (Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));

            for texture in self.textures.values() {
                if texture.0 == set.0 {
                    result.0 = texture.1;
                } else if texture.0 == set.1 {
                    result.1 = texture.1;
                }
            }
        }

        (Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0))
    }

    pub fn get_texture_set_data(&self, id: usize) -> (gl::types::GLuint, gl::types::GLuint) {
        if id < self.texture_sets.len() {
            return self.texture_sets[id];
        }

        (0, 0)
    }

    pub fn get_texture(&mut self, name: &'a str) -> Option<(bool, (gl::types::GLuint, Vec2<f32>))> {
        let mut texture;

        match self.textures.get(name) {
            Some(t) => texture = Some((false, *t)),
            None => texture = None,
        }

        match texture {
            Some(_) => (),
            None => {
                self.load_texture(name);

                match self.textures.get(name) {
                    Some(t) => texture = Some((true, *t)),
                    None => texture = None,
                }
            }
        }

        texture
    }

    pub fn get_cube_map(&self, name: &str) -> Option<gl::types::GLuint> {
        match self.cube_maps.get(name) {
            Some(cube_map) => Some(*cube_map),
            None => None,
        }
    }

    pub unsafe fn clear_all(&mut self) {
        self.clear_all_textures();
        self.clear_all_cube_maps();
    }
    pub unsafe fn clear_all_textures(&mut self) {
        for (_, texture) in self.textures.iter_mut() {
            gl::DeleteTextures(1, &texture.0);
        }

        self.textures.clear();
    }

    pub unsafe fn clear_all_cube_maps(&mut self) {
        for (_, texture) in self.cube_maps.iter_mut() {
            gl::DeleteTextures(1, texture);
        }

        self.cube_maps.clear();
    }

    pub fn load_texture(&mut self, name: &'a str) {
        let image = match lodepng::decode32_file(name) {
            Err(_) => panic!("Failed to load png '{}'!", name),
            Ok(i) => i,
        };

        let mut texture: gl::types::GLuint = 0;

        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                image.width as i32,
                image.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::mem::transmute(image.buffer.as_rgb().as_bytes().as_ptr()),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }

        self.textures.insert(
            name,
            (texture, Vec2::new(image.width as f32, image.height as f32)),
        );
    }

    pub fn get_texture_set(&mut self, albedo: &'a str, emissive: &'a str) -> (bool, usize) {
        let a = self.get_texture(albedo).unwrap().1;
        let e = self.get_texture(emissive).unwrap().1;

        for (index, set) in self.texture_sets.iter().enumerate() {
            if set.0 == a.0 && set.1 == e.0 {
                return (false, index);
            }
        }

        self.texture_sets.push((a.0, e.0));
        (true, self.texture_sets.len() - 1)
    }

    pub fn load_cube_map(&mut self, name: &'a str, files: [&'a str; 6]) {
        let mut texture: gl::types::GLuint = 0;

        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);

            for i in 0..6 {
                let image = match lodepng::decode32_file(files[i]) {
                    Err(_) => panic!("Failed to load png '{}'!", files[i]),
                    Ok(i) => i,
                };

                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as gl::types::GLuint,
                    0,
                    gl::RGBA as i32,
                    image.width as i32,
                    image.height as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    std::mem::transmute(image.buffer.as_rgb().as_bytes().as_ptr()),
                );
            }

            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as i32,
            );
        }

        self.cube_maps.insert(name, texture);
    }
}

impl<'a> Drop for TextureManager<'a> {
    fn drop(&mut self) {
        unsafe {
            self.clear_all_textures();
            self.clear_all_cube_maps();
        };
    }
}
