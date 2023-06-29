use std::f32;

use super::super::Entity;
use super::health::HealthSystem;
use super::transformation::TransformationSystem;
use fnv::FnvHashMap;
use gamemath::Vec3;

pub struct RigidBodySystem {
    timer: (f32, f32),
    gravity: Vec3<f32>,
    map: FnvHashMap<Entity, usize>,
    rigid_bodies: Vec<RigidBody>,
}

pub struct CollisionManifold {
    penetration: f32,
    normal: Vec3<f32>,
}

pub struct RigidBodyBuilder {
    offset: Option<Vec3<f32>>,
    extents: Option<Vec3<f32>>,
    velocity: Option<Vec3<f32>>,
    elasticity: Option<f32>,
    inv_mass: Option<f32>,
    gravity_immune: bool,
    damage: Option<f32>,
    die_on_collision: bool,
}

struct RigidBody {
    owner: Entity,
    offset: Vec3<f32>,
    extents: Vec3<f32>,
    velocity: Vec3<f32>,
    locomotion: Vec3<f32>,
    elasticity: f32,
    inv_mass: f32,
    gravity_immune: bool,
    foothold: bool,
    damage: f32,
    die_on_collision: bool,
}

impl RigidBody {
    pub fn colliding(
        &self,
        other: &RigidBody,
        positions: (Vec3<f32>, Vec3<f32>),
    ) -> Option<CollisionManifold> {
        let _s_min = positions.0 - self.extents;
        let _s_max = positions.0 + self.extents;
        let _o_min = positions.1 - other.extents;
        let _o_max = positions.1 + other.extents;
        let direction = positions.1 - positions.0;
        let overlap = Vec3::new(
            self.extents.x + other.extents.x - direction.x.abs(),
            self.extents.y + other.extents.y - direction.y.abs(),
            self.extents.z + other.extents.z - direction.z.abs(),
        );

        if overlap.x > 0.0 && overlap.y > 0.0 && overlap.z > 0.0 {
            let mut manifold = CollisionManifold {
                penetration: overlap.x.min(overlap.y.min(overlap.z)),
                normal: Vec3::new(0.0, 0.0, 0.0),
            };

            if manifold.penetration == overlap.x {
                if direction.x < 0.0 {
                    manifold.normal = Vec3::new(-1.0, 0.0, 0.0);
                } else {
                    manifold.normal = Vec3::new(1.0, 0.0, 0.0);
                }
            } else if manifold.penetration == overlap.y {
                if direction.y < 0.0 {
                    manifold.normal = Vec3::new(0.0, -1.0, 0.0);
                } else {
                    manifold.normal = Vec3::new(0.0, 1.0, 0.0);
                }
            } else {
                if direction.z < 0.0 {
                    manifold.normal = Vec3::new(0.0, 0.0, -1.0);
                } else {
                    manifold.normal = Vec3::new(0.0, 0.0, 1.0);
                }
            }

            Some(manifold)
        } else {
            None
        }
    }
}

impl RigidBodyBuilder {
    pub fn new() -> RigidBodyBuilder {
        RigidBodyBuilder {
            offset: None,
            extents: None,
            velocity: None,
            elasticity: None,
            inv_mass: None,
            gravity_immune: false,
            damage: None,
            die_on_collision: false,
        }
    }

    pub fn with_offset(mut self, offset: Vec3<f32>) -> RigidBodyBuilder {
        self.offset = Some(offset);
        self
    }

    pub fn with_extents(mut self, extents: Vec3<f32>) -> RigidBodyBuilder {
        self.extents = Some(extents);
        self
    }

    pub fn with_velocity(mut self, velocity: Vec3<f32>) -> RigidBodyBuilder {
        self.velocity = Some(velocity);
        self
    }

    pub fn with_elasticity(mut self, elasticity: f32) -> RigidBodyBuilder {
        self.elasticity = Some(elasticity);
        self
    }

    pub fn is_gravity_immune(mut self) -> RigidBodyBuilder {
        self.gravity_immune = true;
        self
    }

    pub fn dealing_damage(mut self, damage: f32) -> RigidBodyBuilder {
        self.damage = Some(damage);
        self
    }

    pub fn dies_on_collision(mut self) -> RigidBodyBuilder {
        self.die_on_collision = true;
        self
    }

    pub fn with_mass(mut self, mass: f32) -> RigidBodyBuilder {
        self.inv_mass = if mass > 0.0 {
            Some(1.0 / mass)
        } else {
            Some(0.0)
        };

        self
    }

    fn build(self, owner: Entity) -> RigidBody {
        RigidBody {
            owner,
            offset: match self.offset {
                Some(o) => o,
                None => Vec3::new(0.0, 0.0, 0.0),
            },
            extents: match self.extents {
                Some(e) => e,
                None => Vec3::new(0.5, 0.5, 0.5),
            },
            velocity: match self.velocity {
                Some(v) => v,
                None => Vec3::new(0.0, 0.0, 0.0),
            },
            locomotion: Vec3::new(0.0, 0.0, 0.0),
            elasticity: match self.elasticity {
                Some(e) => e,
                None => 0.0,
            },
            inv_mass: match self.inv_mass {
                Some(m) => m,
                None => 0.0,
            },
            gravity_immune: self.gravity_immune,
            foothold: false,
            damage: match self.damage {
                Some(d) => d,
                None => 0.0,
            },
            die_on_collision: self.die_on_collision,
        }
    }
}

impl RigidBodySystem {
    pub fn new() -> RigidBodySystem {
        RigidBodySystem {
            timer: (0.0, 1.0 / 60.0),
            gravity: Vec3::new(0.0, -9.82, 0.0),
            map: FnvHashMap::with_capacity_and_hasher(1, Default::default()),
            rigid_bodies: Vec::new(),
        }
    }

    pub fn add_rigid_body_to_entity(
        &mut self,
        entity: &Entity,
        collider_builder: RigidBodyBuilder,
        transformation_system: &TransformationSystem,
    ) {
        match self.map.contains_key(entity) {
            true => (), //TODO: Add error logging/printing here!
            false => {
                match transformation_system.entity_has_transformation(entity) {
                    true => {
                        self.rigid_bodies.push(collider_builder.build(*entity));
                        self.map.insert(entity.clone(), self.rigid_bodies.len() - 1);
                    }
                    false => (), //TODO: Add error logging/printing here!
                }
            }
        }
    }

    pub fn remove_rigid_body_from_entity(&mut self, entity: &Entity) {
        let mut swapped = (false, 0);
        let mut removed = false;

        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.rigid_bodies.swap_remove(*index);
                    removed = true;

                    if self.rigid_bodies.is_empty() == false {
                        swapped = (true, *index);
                    }
                }
                None => (),
            }
        }

        if removed == true {
            self.map.remove(entity);
        }

        if swapped.0 == true && swapped.1 != self.rigid_bodies.len() {
            *self
                .map
                .get_mut(&self.rigid_bodies[swapped.1].owner)
                .unwrap() = swapped.1;
        }
    }

    pub fn entity_has_rigid_body(&self, entity: &Entity) -> bool {
        self.map.contains_key(entity)
    }

    pub fn get_extents(&self, entity: &Entity) -> Option<Vec3<f32>> {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => Some(self.rigid_bodies[*index].extents),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn set_locomotion(&mut self, entity: &Entity, locomotion: Vec3<f32>) {
        match self.map.get(entity) {
            Some(index) => {
                self.rigid_bodies[*index].locomotion = locomotion;
            }
            None => (),
        }
    }

    pub fn set_gravity_immunity(&mut self, entity: &Entity, immunity: bool) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => self.rigid_bodies[*index].gravity_immune = immunity,
                None => (),
            }
        }
    }

    pub fn apply_force(&mut self, entity: &Entity, force: Vec3<f32>) {
        match self.map.get(entity) {
            Some(index) => {
                self.rigid_bodies[*index].velocity += force;
            }
            None => (),
        }
    }

    pub fn apply_flight_force(&mut self, entity: &Entity, force: f32) {
        match self.map.get(entity) {
            Some(index) => {
                let mut velocity = &mut self.rigid_bodies[*index].velocity;

                if velocity.y < force {
                    velocity.y = force;
                }
            }
            None => (),
        }
    }

    pub fn set_velocity(&mut self, entity: &Entity, velocity: Vec3<f32>) {
        match self.map.get(entity) {
            Some(index) => {
                self.rigid_bodies[*index].velocity = velocity;
            }
            None => (),
        }
    }

    pub fn entity_has_foothold(&self, entity: &Entity) -> bool {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => self.rigid_bodies[*index].foothold,
                None => false,
            }
        } else {
            false
        }
    }

    //ray(origin, direction)
    pub fn ray_cast(
        &self,
        ray: (Vec3<f32>, Vec3<f32>),
        transformation_system: &TransformationSystem,
        user: Entity,
    ) -> Option<(f32, Entity)> {
        let inv = Vec3::new(1.0 / ray.1.x, 1.0 / ray.1.y, 1.0 / ray.1.z);
        let mut result = None;

        for collider in self.rigid_bodies.iter() {
            let aabb = (
                transformation_system.get_position(&collider.owner).unwrap(),
                collider.extents,
            );

            if collider.owner != user
                && RigidBodySystem::ray_vs_aabb_intersecting(&aabb, (&ray.0, &ray.1, &inv)) == true
            {
                match result {
                    None => result = Some(((aabb.0 - ray.0).length_squared(), collider.owner)),
                    Some(r) => {
                        let distance = (aabb.0 - ray.0).length_squared();

                        if distance < r.0 {
                            result = Some((distance, collider.owner));
                        }
                    }
                }
            }
        }

        result
    }

    //aabb: (position, extents), ray: (origin, direction, direction_inverse)
    pub fn ray_vs_aabb_intersecting(
        aabb: &(Vec3<f32>, Vec3<f32>),
        ray: (&Vec3<f32>, &Vec3<f32>, &Vec3<f32>),
    ) -> bool {
        let t1 = ((aabb.0.x - aabb.1.x) - ray.0.x) * ray.2.x;
        let t2 = ((aabb.0.x + aabb.1.x) - ray.0.x) * ray.2.x;
        let tmin = t1.min(t2);
        let tmax = t1.max(t2);

        for i in 1..3 {
            let t1 = ((aabb.0[i] - aabb.1[i]) - ray.0[i]) * ray.2[i];
            let t2 = ((aabb.0[i] + aabb.1[i]) - ray.0[i]) * ray.2[i];
            let _tmin = tmin.max(t1.min(t2));
            let _tmax = tmax.min(t1.max(t2));
        }

        tmax > tmin.max(0.0)
    }

    pub fn update_colliders(
        &mut self,
        first: usize,
        last: usize,
        transformation_system: &mut TransformationSystem,
    ) {
        for i in first..last {
            let mut collider = &mut self.rigid_bodies[i];
            let pos = transformation_system
                .get_position_mut(&collider.owner)
                .unwrap();

            if collider.inv_mass > 0.0 && collider.gravity_immune == false {
                collider.velocity += self.gravity * self.timer.1;
            }

            *pos += (collider.velocity + collider.locomotion) * self.timer.1;
            collider.locomotion = Vec3::new(0.0, 0.0, 0.0);
            collider.foothold = false;
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        transformation_system: &mut TransformationSystem,
        health_system: &mut HealthSystem,
    ) {
        self.timer.0 += dt;

        let count = self.rigid_bodies.len();
        let mut split = None;

        if count >= 2 {
            split = Some((0, count / 2));
        }

        while self.timer.0 >= self.timer.1 && count > 0 {
            self.timer.0 -= self.timer.1;

            //UPDATE HERE
            //match split {
            //    None => self.update_colliders(0, count, transformation_system),
            //    Some((first_start, second_start)) => {
            //        //let t1 = thread::spawn(self.update_colliders);
            //        let t1 = thread::spawn();
            //        t1.join():
            //    },
            //}
            self.update_colliders(0, count, transformation_system);
            //

            for i in 0..(self.rigid_bodies.len() - 1) {
                let position_1 = transformation_system
                    .get_position(&self.rigid_bodies[i].owner)
                    .unwrap()
                    + self.rigid_bodies[i].offset;

                for j in (i + 1)..self.rigid_bodies.len() {
                    if self.rigid_bodies[i].inv_mass != 0.0 || self.rigid_bodies[j].inv_mass != 0.0
                    {
                        let position_2 = transformation_system
                            .get_position(&self.rigid_bodies[j].owner)
                            .unwrap()
                            + self.rigid_bodies[j].offset;

                        match self.rigid_bodies[i]
                            .colliding(&self.rigid_bodies[j], (position_1, position_2))
                        {
                            Some(manifold) => {
                                if self.rigid_bodies[i].inv_mass == 0.0
                                    && manifold.normal == Vec3::new(0.0, 1.0, 0.0)
                                {
                                    self.rigid_bodies[j].foothold = true;
                                } else if self.rigid_bodies[j].inv_mass == 0.0
                                    && manifold.normal == Vec3::new(0.0, -1.0, 0.0)
                                {
                                    self.rigid_bodies[i].foothold = true;
                                }

                                let rv =
                                    self.rigid_bodies[j].velocity - self.rigid_bodies[i].velocity;
                                let normal_vel = rv.dot(manifold.normal);
                                let masses =
                                    (self.rigid_bodies[i].inv_mass, self.rigid_bodies[j].inv_mass);

                                if normal_vel > 0.0 {
                                    continue;
                                }

                                let e = self.rigid_bodies[i]
                                    .elasticity
                                    .max(self.rigid_bodies[j].elasticity);

                                let mut normal_magnitude = -(1.0 + e) * normal_vel;
                                normal_magnitude /= masses.0 + masses.1;

                                let impulse = manifold.normal * normal_magnitude;

                                self.rigid_bodies[i].velocity -= impulse * masses.0;
                                self.rigid_bodies[j].velocity += impulse * masses.1;

                                let mass_factor = 1.0 / (masses.0 + masses.1);
                                let corrections = (
                                    manifold.normal * mass_factor * masses.0 * manifold.penetration,
                                    manifold.normal * mass_factor * masses.1 * manifold.penetration,
                                );

                                *transformation_system
                                    .get_position_mut(&self.rigid_bodies[i].owner)
                                    .unwrap() -= corrections.0;
                                *transformation_system
                                    .get_position_mut(&self.rigid_bodies[j].owner)
                                    .unwrap() += corrections.1;

                                if health_system.entity_has_health(&self.rigid_bodies[i].owner)
                                    == true
                                {
                                    health_system.harm(
                                        &self.rigid_bodies[i].owner,
                                        self.rigid_bodies[j].damage,
                                    );

                                    if self.rigid_bodies[i].die_on_collision == true {
                                        health_system.kill_entity(&self.rigid_bodies[i].owner);
                                    }
                                }

                                if health_system.entity_has_health(&self.rigid_bodies[j].owner)
                                    == true
                                {
                                    health_system.harm(
                                        &self.rigid_bodies[j].owner,
                                        self.rigid_bodies[i].damage,
                                    );

                                    if self.rigid_bodies[j].die_on_collision == true {
                                        health_system.kill_entity(&self.rigid_bodies[j].owner);
                                    }
                                }
                            }
                            None => (),
                        }
                    }
                }
            }
        }
    }
}
