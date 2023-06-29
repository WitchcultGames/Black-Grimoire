

use fnv::FnvHashMap;
use super::super::{Entity, EntityManager};
use super::transformation::{TransformationSystem, TransformationBuilder};
use super::drawable::{DrawableSystem, DrawableBuilder};
use super::rigid_body::{RigidBodySystem, RigidBodyBuilder};
use super::super::super::renderer::Renderer;
use super::super::super::range::Range;
use gamemath::Vec3;
use gameprng::xorshift128plus::XorShift128Plus;
use gameprng::prng_traits::PrngGeneration;

pub struct ParticleEmitterData {
    owner: Entity,
    particle_lifetime: Range,
    particle_velocity: (Range, Range, Range),
    emission_timer: (f32, f32),
    particles: Vec<(Entity, f32, f32)>,
}

pub struct ParticleEmitterSystem {
    map: FnvHashMap<Entity, usize>,
    data: Vec<ParticleEmitterData>,
}

pub struct ParticleEmitterBuilder {
}

impl ParticleEmitterBuilder {
    pub fn new() -> ParticleEmitterBuilder {
        ParticleEmitterBuilder {
        }
    }

    fn build(self,
             owner: Entity,
             renderer: &mut Renderer,
             entity_manager: &mut EntityManager,
             transformation_system: &mut TransformationSystem,
             rigid_body_system: &mut RigidBodySystem,
             drawable_system: &mut DrawableSystem) -> ParticleEmitterData {
        let mut pe = ParticleEmitterData {
            owner,
            particle_lifetime: Range::new(0.25, 0.5),
            particle_velocity: (Range::new(-0.25, 0.25),
                                Range::new(-0.25, 0.25),
                                Range::new(-0.25, 0.25)),
            emission_timer: (0.0, 1.0 / 50.0),
            particles: Vec::new(),
        };

        let count = ((pe.particle_lifetime.get_max() * (1.0 / pe.emission_timer.1)) + 0.5) as usize;
        pe.particles.reserve(count);


        for _ in 0..count {
            let p = entity_manager.create_new_entity();
            entity_manager.set_entity_is_active(&p, false);

            transformation_system.add_transformation_to_entity(&p,
                                                               TransformationBuilder::new()
                                                                   .with_scale(Vec3::new(0.015625,
                                                                                         0.015625,
                                                                                         0.015625)));

            rigid_body_system.add_rigid_body_to_entity(&p,
                                                       RigidBodyBuilder::new()
                                                           .with_extents(Vec3::new(0.0078125,
                                                                                   0.0078125,
                                                                                   0.0078125))
                                                           .with_mass(0.0001)
                                                           .with_elasticity(0.25)
                                                           .is_gravity_immune(),
                                                       transformation_system);

            drawable_system.add_drawable_to_entity(&p,
                                                   transformation_system,
                                                   renderer,
                                                   DrawableBuilder::new()
                                                       .using_shader("test")
                                                       .using_model("cube")
                                                       .using_texture_set("box.png", "black.png"));

            pe.particles.push((p, 0.0, 0.0));
        }

        pe
    }
}

impl ParticleEmitterData {
    pub fn emit(&mut self,
                _lifetime: f32,
                _position: Vec3<f32>,
                _velocity: Vec3<f32>,
                _entity_manager: &mut EntityManager,
                _transformation_system: &mut TransformationSystem,
                _rigid_body_system: &mut RigidBodySystem) {
    }

    pub fn update(&mut self,
                  dt: f32,
                  prng: &mut XorShift128Plus,
                  _renderer: &mut Renderer,
                  entity_manager: &mut EntityManager,
                  transformation_system: &mut TransformationSystem,
                  rigid_body_system: &mut RigidBodySystem,
                  _drawable_system: &mut DrawableSystem) {
        self.emission_timer.0 += dt;

        for particle in self.particles.iter_mut() {
            if entity_manager.entity_is_active(&particle.0) == true {
                particle.1 += dt;

                if particle.1 >= particle.2 {
                    entity_manager.set_entity_is_active(&particle.0, false);
                }
            }
        }

        while self.emission_timer.0 >= self.emission_timer.1 {
            self.emission_timer.0 -= self.emission_timer.1;

            for particle in self.particles.iter_mut() {
                if entity_manager.entity_is_active(&particle.0) == false {
                    let t = prng.range(self.particle_lifetime.get_min(), self.particle_lifetime.get_max());
                    let p = transformation_system.get_position(&self.owner).unwrap();
                    let v = Vec3::new(prng.range(self.particle_velocity.0.get_min(), self.particle_velocity.0.get_max()),
                                      prng.range(self.particle_velocity.1.get_min(), self.particle_velocity.1.get_max()),
                                      prng.range(self.particle_velocity.2.get_min(), self.particle_velocity.2.get_max()));

                    rigid_body_system.set_velocity(&particle.0, v);
                    transformation_system.set_position(&particle.0, p);
                    entity_manager.set_entity_is_active(&particle.0, true);
                    particle.1 = 0.0;
                    particle.2 = t;
                    break;
                }
            }
        }
    }
}

impl ParticleEmitterSystem {
    pub fn new() -> ParticleEmitterSystem {
        ParticleEmitterSystem {
            map: FnvHashMap::with_capacity_and_hasher(1, Default::default()),
            data: Vec::new(),
        }
    }

    pub fn add_particle_emitter_to_entity(&mut self,
                                          entity: &Entity,
                                          init_data: ParticleEmitterBuilder,
                                          renderer: &mut Renderer,
                                          entity_manager: &mut EntityManager,
                                          transformation_system: &mut TransformationSystem,
                                          rigid_body_system: &mut RigidBodySystem,
                                          drawable_system: &mut DrawableSystem) {
        match self.entity_has_particle_emitter(entity) {
            true => (), //TODO: Add error logging/printing here!
            false => {
                self.data.push(init_data.build(*entity,
                                               renderer,
                                               entity_manager,
                                               transformation_system,
                                               rigid_body_system,
                                               drawable_system));

                self.map.insert(entity.clone(), self.data.len() - 1);
            },
        }
    }

    pub fn remove_particle_emitter_from_entity(&mut self,
                                               entity: &Entity,
                                               entity_manager: &mut EntityManager) {
        let mut swapped = (false, 0);
        let mut removed = false;

        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    for particle in self.data[*index].particles.iter() {
                        entity_manager.destroy_entity(&particle.0);
                    }

                    self.data.swap_remove(*index);
                    removed = true;

                    if self.data.is_empty() == false {
                        swapped = (true, *index);
                    }
                },
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

    pub fn update(&mut self,
                  dt: f32,
                  prng: &mut XorShift128Plus,
                  renderer: &mut Renderer,
                  entity_manager: &mut EntityManager,
                  transformation_system: &mut TransformationSystem,
                  rigid_body_system: &mut RigidBodySystem,
                  drawable_system: &mut DrawableSystem) {
        for emitter in self.data.iter_mut() {
            if entity_manager.entity_is_active(&emitter.owner) == true {
                emitter.update(dt,
                               prng,
                               renderer,
                               entity_manager,
                               transformation_system,
                               rigid_body_system,
                               drawable_system);
            }
        }
    }

    pub fn entity_has_particle_emitter(&self, entity: &Entity) -> bool {
        self.map.contains_key(entity)
    }
}
