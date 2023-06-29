use super::super::{Entity, EntityManager};
use fnv::FnvHashMap;
use std::f32;

struct HealthData {
    owner: Entity,
    hitpoints: (f32, f32),
}

pub struct HealthSystem {
    map: FnvHashMap<Entity, usize>,
    data: Vec<HealthData>,
}

pub struct HealthBuilder {
    hitpoints: Option<(f32, f32)>,
}

impl HealthBuilder {
    pub fn new() -> HealthBuilder {
        HealthBuilder { hitpoints: None }
    }

    pub fn with_hitpoints(mut self, hitpoints: (f32, f32)) -> HealthBuilder {
        self.hitpoints = Some(hitpoints);
        self
    }

    fn build(self, owner: Entity) -> HealthData {
        HealthData {
            owner,
            hitpoints: match self.hitpoints {
                Some(hp) => hp,
                None => (1.0, 1.0),
            },
        }
    }
}

impl HealthSystem {
    pub fn new() -> HealthSystem {
        HealthSystem {
            map: FnvHashMap::with_capacity_and_hasher(1, Default::default()),
            data: Vec::new(),
        }
    }

    pub fn add_health_to_entity(&mut self, entity: &Entity, initial_health: HealthBuilder) {
        if *entity != Entity::null() {
            match self.entity_has_health(entity) {
                true => (), //TODO: Add error logging/printing here!
                false => {
                    self.data.push(initial_health.build(*entity));
                    self.map.insert(entity.clone(), self.data.len() - 1);
                }
            }
        }
    }

    pub fn remove_health_from_entity(&mut self, entity: &Entity) {
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

    pub fn heal(&mut self, entity: &Entity, amount: f32) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].hitpoints.0 += amount;
                }
                None => (),
            }
        }
    }

    pub fn harm(&mut self, entity: &Entity, amount: f32) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].hitpoints.0 -= amount;
                }
                None => (),
            }
        }
    }

    pub fn kill_entity(&mut self, entity: &Entity) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].hitpoints.0 = f32::MIN;
                }
                None => (),
            }
        }
    }

    pub fn update(&mut self, entity_manager: &mut EntityManager) {
        for health in self.data.iter_mut() {
            if health.hitpoints.0 > health.hitpoints.1 {
                health.hitpoints.0 = health.hitpoints.1;
            } else if health.hitpoints.0 <= 0.0 {
                entity_manager.destroy_entity(&health.owner);
            }
        }
    }

    pub fn entity_has_health(&self, entity: &Entity) -> bool {
        self.map.contains_key(entity)
    }
}
