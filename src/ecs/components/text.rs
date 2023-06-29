use super::super::super::renderer::model::ModelInfo;
use super::super::super::renderer::{RenderJob, Renderer};
use super::super::{Entity, EntityManager};
use super::transformation::TransformationSystem;
use fnv::FnvHashMap;
use gamemath::Vec2;
use gamemath::Vec3;
use gamemath::Vec4;
use gl::types::GLuint;
use std::str::FromStr;

struct TextData {
    owner: Entity,
    shader: GLuint,
    model: ModelInfo,
    texture_set: usize,
    character_size: Vec2<f32>,
    uv_scale: Vec2<f32>,
    tint: Vec4<f32>,
    emissive_tint: Vec4<f32>,
    offset: Vec3<f32>,
    text: String,
}

pub struct TextSystem {
    map: FnvHashMap<Entity, usize>,
    data: Vec<TextData>,
}

pub struct TextBuilder<'a> {
    shader: Option<&'a str>,
    model: Option<&'a str>,
    texture_set: Option<(&'a str, &'a str)>,
    uv_scale: Option<Vec2<f32>>,
    tint: Option<Vec4<f32>>,
    emissive_tint: Option<Vec4<f32>>,
    offset: Option<Vec3<f32>>,
    text: Option<String>,
}

impl<'a> TextBuilder<'a> {
    pub fn new() -> TextBuilder<'a> {
        TextBuilder {
            shader: None,
            model: None,
            texture_set: None,
            uv_scale: None,
            tint: None,
            emissive_tint: None,
            offset: None,
            text: None,
        }
    }

    pub fn using_shader(mut self, shader_name: &'a str) -> TextBuilder {
        self.shader = Some(shader_name);
        self
    }

    pub fn using_model(mut self, model_name: &'a str) -> TextBuilder {
        self.model = Some(model_name);
        self
    }

    pub fn using_texture_set(mut self, albedo: &'a str, emissive: &'a str) -> TextBuilder<'a> {
        self.texture_set = Some((albedo, emissive));
        self
    }

    pub fn with_uv_scale(mut self, uv_scale: Vec2<f32>) -> TextBuilder<'a> {
        self.uv_scale = Some(uv_scale);
        self
    }

    pub fn with_tint(mut self, tint: Vec4<f32>) -> TextBuilder<'a> {
        self.tint = Some(tint);
        self
    }

    pub fn with_emissive_tint(mut self, tint: Vec4<f32>) -> TextBuilder<'a> {
        self.emissive_tint = Some(tint);
        self
    }

    pub fn with_text(mut self, text: String) -> TextBuilder<'a> {
        self.text = Some(text);
        self
    }

    pub fn with_offset(mut self, offset: Vec3<f32>) -> TextBuilder<'a> {
        self.offset = Some(offset);
        self
    }

    fn build(self, owner: Entity, renderer: &mut Renderer<'a>) -> TextData {
        let mut new_text = TextData {
            owner,
            shader: match self.shader {
                Some(s) => renderer.get_shader(s).unwrap(),
                None => renderer.get_shader("test").unwrap(),
            },
            model: match self.model {
                Some(m) => renderer.get_model(m).unwrap(),
                None => renderer.get_model("cube").unwrap(),
            },
            texture_set: match self.texture_set {
                Some(t) => renderer.get_texture_set(t.0, t.1),
                None => renderer.get_texture_set("font.png", "black.png"),
            },
            character_size: Vec2::new(0.0, 0.0),
            uv_scale: match self.uv_scale {
                Some(s) => s,
                None => Vec2::new(1.0, 1.0),
            },
            tint: match self.tint {
                Some(t) => t,
                None => Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            emissive_tint: match self.emissive_tint {
                Some(t) => t,
                None => Vec4::new(0.0, 0.0, 0.0, 0.0),
            },
            offset: match self.offset {
                Some(o) => o,
                None => Vec3::new(0.0, 0.0, 0.0),
            },
            text: match self.text {
                Some(t) => t,
                None => String::from_str("Text").unwrap(),
            },
        };

        let sizes = renderer.get_texture_set_sizes(new_text.texture_set);

        new_text.character_size = Vec2::new(sizes.0.x / 10.0, sizes.0.y / 10.0);

        new_text.character_size = Vec2::new(6.0, 6.0);

        new_text
    }
}

impl<'a> TextSystem {
    pub fn new() -> TextSystem {
        TextSystem {
            map: FnvHashMap::with_capacity_and_hasher(1, Default::default()),
            data: Vec::new(),
        }
    }

    pub fn add_text_to_entity(
        &mut self,
        entity: &Entity,
        transformation_system: &TransformationSystem,
        renderer: &mut Renderer<'a>,
        initial_data: TextBuilder<'a>,
    ) {
        match self.map.contains_key(entity) {
            true => (), //TODO: Add error logging/printing here!
            false => {
                match transformation_system.entity_has_transformation(entity) {
                    true => {
                        self.data.push(initial_data.build(*entity, renderer));
                        self.map.insert(entity.clone(), self.data.len() - 1);
                    }
                    false => (), //TODO: Add error logging/printing here!
                }
            }
        }
    }

    pub fn remove_text_from_entity(&mut self, entity: &Entity) {
        let mut swapped = (false, 0);
        let mut removed = false;

        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data.swap_remove(*index);
                    removed = true;

                    if self.data.is_empty() == false {
                        swapped = (true, *index);
                    }
                }
                None => (),
            }
        }

        if removed == true {
            self.map.remove(entity);
        }

        if swapped.0 == true && swapped.1 != self.data.len() {
            *self.map.get_mut(&self.data[swapped.1].owner).unwrap() = swapped.1;
        }
    }

    pub fn entity_has_text(&self, entity: &Entity) -> bool {
        self.map.contains_key(entity)
    }

    pub fn set_tint_color(&mut self, entity: &Entity, color: Vec4<f32>) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].tint = color;
                }
                None => (),
            }
        }
    }

    pub fn set_text(&mut self, entity: &Entity, text: &str) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].text = String::from_str(text).unwrap();
                }
                None => (),
            }
        }
    }

    pub fn draw_all(
        &self,
        entity_manager: &EntityManager,
        transformation_system: &TransformationSystem,
        renderer: &mut Renderer,
    ) {
        for text in self.data.iter() {
            if entity_manager.entity_is_active(&text.owner) == true {
                let t = transformation_system
                    .get_transformation_data(&text.owner)
                    .unwrap();
                let uv_size = Vec2::new(0.1, 0.1);
                let mut uv = Vec2::new(0.0, 0.0);
                let mut character_position = t.position;

                for c in text.text.chars() {
                    match c {
                        'a' => uv = Vec2::new(uv_size.x * 0.0, uv_size.y * 1.0),
                        'b' => uv = Vec2::new(uv_size.x * 1.0, uv_size.y * 1.0),
                        'c' => uv = Vec2::new(uv_size.x * 2.0, uv_size.y * 1.0),
                        'd' => uv = Vec2::new(uv_size.x * 3.0, uv_size.y * 1.0),
                        'e' => uv = Vec2::new(uv_size.x * 4.0, uv_size.y * 1.0),
                        'f' => uv = Vec2::new(uv_size.x * 5.0, uv_size.y * 1.0),
                        'g' => uv = Vec2::new(uv_size.x * 6.0, uv_size.y * 1.0),
                        'h' => uv = Vec2::new(uv_size.x * 7.0, uv_size.y * 1.0),
                        'i' => uv = Vec2::new(uv_size.x * 8.0, uv_size.y * 1.0),
                        'j' => uv = Vec2::new(uv_size.x * 9.0, uv_size.y * 1.0),
                        'k' => uv = Vec2::new(uv_size.x * 0.0, uv_size.y * 2.0),
                        'l' => uv = Vec2::new(uv_size.x * 1.0, uv_size.y * 2.0),
                        'm' => uv = Vec2::new(uv_size.x * 2.0, uv_size.y * 2.0),
                        'n' => uv = Vec2::new(uv_size.x * 3.0, uv_size.y * 2.0),
                        'o' => uv = Vec2::new(uv_size.x * 4.0, uv_size.y * 2.0),
                        'p' => uv = Vec2::new(uv_size.x * 5.0, uv_size.y * 2.0),
                        'q' => uv = Vec2::new(uv_size.x * 6.0, uv_size.y * 2.0),
                        'r' => uv = Vec2::new(uv_size.x * 7.0, uv_size.y * 2.0),
                        's' => uv = Vec2::new(uv_size.x * 8.0, uv_size.y * 2.0),
                        't' => uv = Vec2::new(uv_size.x * 9.0, uv_size.y * 2.0),
                        'u' => uv = Vec2::new(uv_size.x * 0.0, uv_size.y * 3.0),
                        'v' => uv = Vec2::new(uv_size.x * 1.0, uv_size.y * 3.0),
                        'w' => uv = Vec2::new(uv_size.x * 2.0, uv_size.y * 3.0),
                        'x' => uv = Vec2::new(uv_size.x * 3.0, uv_size.y * 3.0),
                        'y' => uv = Vec2::new(uv_size.x * 4.0, uv_size.y * 3.0),
                        'z' => uv = Vec2::new(uv_size.x * 5.0, uv_size.y * 3.0),
                        ',' => uv = Vec2::new(uv_size.x * 6.0, uv_size.y * 3.0),
                        '.' => uv = Vec2::new(uv_size.x * 7.0, uv_size.y * 3.0),
                        ':' => uv = Vec2::new(uv_size.x * 8.0, uv_size.y * 3.0),
                        ';' => uv = Vec2::new(uv_size.x * 9.0, uv_size.y * 3.0),
                        'A' => uv = Vec2::new(uv_size.x * 0.0, uv_size.y * 4.0),
                        'B' => uv = Vec2::new(uv_size.x * 1.0, uv_size.y * 4.0),
                        'C' => uv = Vec2::new(uv_size.x * 2.0, uv_size.y * 4.0),
                        'D' => uv = Vec2::new(uv_size.x * 3.0, uv_size.y * 4.0),
                        'E' => uv = Vec2::new(uv_size.x * 4.0, uv_size.y * 4.0),
                        'F' => uv = Vec2::new(uv_size.x * 5.0, uv_size.y * 4.0),
                        'G' => uv = Vec2::new(uv_size.x * 6.0, uv_size.y * 4.0),
                        'H' => uv = Vec2::new(uv_size.x * 7.0, uv_size.y * 4.0),
                        'I' => uv = Vec2::new(uv_size.x * 8.0, uv_size.y * 4.0),
                        'J' => uv = Vec2::new(uv_size.x * 9.0, uv_size.y * 4.0),
                        'K' => uv = Vec2::new(uv_size.x * 0.0, uv_size.y * 5.0),
                        'L' => uv = Vec2::new(uv_size.x * 1.0, uv_size.y * 5.0),
                        'M' => uv = Vec2::new(uv_size.x * 2.0, uv_size.y * 5.0),
                        'N' => uv = Vec2::new(uv_size.x * 3.0, uv_size.y * 5.0),
                        'O' => uv = Vec2::new(uv_size.x * 4.0, uv_size.y * 5.0),
                        'P' => uv = Vec2::new(uv_size.x * 5.0, uv_size.y * 5.0),
                        'Q' => uv = Vec2::new(uv_size.x * 6.0, uv_size.y * 5.0),
                        'R' => uv = Vec2::new(uv_size.x * 7.0, uv_size.y * 5.0),
                        'S' => uv = Vec2::new(uv_size.x * 8.0, uv_size.y * 5.0),
                        'T' => uv = Vec2::new(uv_size.x * 9.0, uv_size.y * 5.0),
                        'U' => uv = Vec2::new(uv_size.x * 0.0, uv_size.y * 6.0),
                        'V' => uv = Vec2::new(uv_size.x * 1.0, uv_size.y * 6.0),
                        'W' => uv = Vec2::new(uv_size.x * 2.0, uv_size.y * 6.0),
                        'X' => uv = Vec2::new(uv_size.x * 3.0, uv_size.y * 6.0),
                        'Y' => uv = Vec2::new(uv_size.x * 4.0, uv_size.y * 6.0),
                        'Z' => uv = Vec2::new(uv_size.x * 5.0, uv_size.y * 6.0),
                        '!' => uv = Vec2::new(uv_size.x * 6.0, uv_size.y * 6.0),
                        '?' => uv = Vec2::new(uv_size.x * 7.0, uv_size.y * 6.0),
                        '\'' => uv = Vec2::new(uv_size.x * 8.0, uv_size.y * 6.0),
                        '"' => uv = Vec2::new(uv_size.x * 9.0, uv_size.y * 6.0),
                        '0' => uv = Vec2::new(uv_size.x * 0.0, uv_size.y * 7.0),
                        '1' => uv = Vec2::new(uv_size.x * 1.0, uv_size.y * 7.0),
                        '2' => uv = Vec2::new(uv_size.x * 2.0, uv_size.y * 7.0),
                        '3' => uv = Vec2::new(uv_size.x * 3.0, uv_size.y * 7.0),
                        '4' => uv = Vec2::new(uv_size.x * 4.0, uv_size.y * 7.0),
                        '5' => uv = Vec2::new(uv_size.x * 5.0, uv_size.y * 7.0),
                        '6' => uv = Vec2::new(uv_size.x * 6.0, uv_size.y * 7.0),
                        '7' => uv = Vec2::new(uv_size.x * 7.0, uv_size.y * 7.0),
                        '8' => uv = Vec2::new(uv_size.x * 8.0, uv_size.y * 7.0),
                        '9' => uv = Vec2::new(uv_size.x * 9.0, uv_size.y * 7.0),
                        '-' => uv = Vec2::new(uv_size.x * 0.0, uv_size.y * 8.0),
                        '+' => uv = Vec2::new(uv_size.x * 1.0, uv_size.y * 8.0),
                        '%' => uv = Vec2::new(uv_size.x * 2.0, uv_size.y * 8.0),
                        '\n' => {
                            character_position.x = t.position.x;
                            character_position.y -= text.character_size.y + 1.0;
                            continue;
                        }
                        ' ' => {
                            character_position.x += text.character_size.x * 2.0 + 2.0;
                            continue;
                        }
                        _ => continue,
                    }

                    renderer.add_render_job(RenderJob {
                        model: text.model,
                        shader: text.shader,
                        textures: text.texture_set,
                        scale: Vec3::new(text.character_size.x, text.character_size.y, 1.0),
                        uv_size,
                        uv_offset: uv,
                        position: character_position + text.offset,
                        pivot: t.pivot,
                        rotation: t.rotation,
                        tint: text.tint,
                        emissive_tint: text.emissive_tint,
                    });

                    character_position.x += text.character_size.x * 2.0 + 2.0;
                }
            }
        }
    }
}
