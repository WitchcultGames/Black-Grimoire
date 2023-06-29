use super::super::super::renderer::model::ModelInfo;
use super::super::super::renderer::{RenderJob, Renderer};
use super::super::{Entity, EntityManager};
use super::transformation::TransformationSystem;
use fnv::FnvHashMap;
use gamemath::Vec2;
use gamemath::Vec4;
use gl::types::GLuint;

struct DrawableData {
    owner: Entity,
    shader: GLuint,
    model: ModelInfo,
    texture_set: usize,
    uv_scale: Vec2<f32>,
    uv_offset: Vec2<f32>,
    tint: Vec4<f32>,
    emissive_tint: Vec4<f32>,
}

pub struct DrawableSystem {
    map: FnvHashMap<Entity, usize>,
    data: Vec<DrawableData>,
}

pub struct DrawableBuilder<'a> {
    shader: Option<&'a str>,
    model: Option<&'a str>,
    texture_set: Option<(&'a str, &'a str)>,
    uv_scale: Option<Vec2<f32>>,
    uv_offset: Option<Vec2<f32>>,
    tint: Option<Vec4<f32>>,
    emissive_tint: Option<Vec4<f32>>,
}

impl<'a> DrawableBuilder<'a> {
    pub fn new() -> DrawableBuilder<'a> {
        DrawableBuilder {
            shader: None,
            model: None,
            texture_set: None,
            uv_scale: None,
            uv_offset: None,
            tint: None,
            emissive_tint: None,
        }
    }

    pub fn using_shader(mut self, shader_name: &'a str) -> DrawableBuilder {
        self.shader = Some(shader_name);
        self
    }

    pub fn using_model(mut self, model_name: &'a str) -> DrawableBuilder {
        self.model = Some(model_name);
        self
    }

    pub fn using_texture_set(mut self, albedo: &'a str, emissive: &'a str) -> DrawableBuilder<'a> {
        self.texture_set = Some((albedo, emissive));
        self
    }

    pub fn with_uv_scale(mut self, uv_scale: Vec2<f32>) -> DrawableBuilder<'a> {
        self.uv_scale = Some(uv_scale);
        self
    }

    pub fn with_uv_offset(mut self, uv_offset: Vec2<f32>) -> DrawableBuilder<'a> {
        self.uv_offset = Some(uv_offset);
        self
    }

    pub fn with_tint(mut self, tint: Vec4<f32>) -> DrawableBuilder<'a> {
        self.tint = Some(tint);
        self
    }

    pub fn with_emissive_tint(mut self, tint: Vec4<f32>) -> DrawableBuilder<'a> {
        self.emissive_tint = Some(tint);
        self
    }

    fn build(self, owner: Entity, renderer: &mut Renderer<'a>) -> DrawableData {
        DrawableData {
            owner,
            shader: match self.shader {
                Some(s) => renderer.get_shader(s).unwrap(),
                None => renderer.get_shader("sprite").unwrap(),
            },
            model: match self.model {
                Some(m) => renderer.get_model(m).unwrap(),
                None => renderer.get_model("cube").unwrap(),
            },
            texture_set: match self.texture_set {
                Some(t) => renderer.get_texture_set(t.0, t.1),
                None => renderer.get_texture_set("pixel.png", "black.png"),
            },
            uv_scale: match self.uv_scale {
                Some(s) => s,
                None => Vec2::new(1.0, 1.0),
            },
            uv_offset: match self.uv_offset {
                Some(o) => o,
                None => Vec2::new(0.0, 0.0),
            },
            tint: match self.tint {
                Some(t) => t,
                None => Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            emissive_tint: match self.emissive_tint {
                Some(t) => t,
                None => Vec4::new(0.0, 0.0, 0.0, 0.0),
            },
        }
    }
}

impl<'a> DrawableSystem {
    pub fn new() -> DrawableSystem {
        DrawableSystem {
            map: FnvHashMap::with_capacity_and_hasher(1, Default::default()),
            data: Vec::new(),
        }
    }

    pub fn add_drawable_to_entity(
        &mut self,
        entity: &Entity,
        transformation_system: &TransformationSystem,
        renderer: &mut Renderer<'a>,
        initial_data: DrawableBuilder<'a>,
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

    pub fn remove_drawable_from_entity(&mut self, entity: &Entity) {
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

    pub fn entity_has_drawable(&self, entity: &Entity) -> bool {
        self.map.contains_key(entity)
    }

    pub fn set_entity_tint_color(&mut self, entity: &Entity, color: Vec4<f32>) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].tint = color;
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
        for drawable in self.data.iter() {
            if entity_manager.entity_is_active(&drawable.owner) == true {
                let t = transformation_system
                    .get_transformation_data(&drawable.owner)
                    .unwrap();

                renderer.add_render_job(RenderJob {
                    model: drawable.model,
                    shader: drawable.shader,
                    textures: drawable.texture_set,
                    scale: t.scale,
                    uv_size: drawable.uv_scale,
                    uv_offset: drawable.uv_offset,
                    position: t.position,
                    pivot: t.pivot,
                    rotation: t.rotation,
                    tint: drawable.tint,
                    emissive_tint: drawable.emissive_tint,
                });
            }
        }
    }
}
