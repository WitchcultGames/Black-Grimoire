use std::collections::VecDeque;

pub mod components;

// DO NOT CHANGE BELOW!
const ENTITY_INDEX_BITS: u32 = 24;
const ENTITY_GENERATION_BITS: u32 = 8;
const ENTITY_INDEX_MASK: u32 = (1 << ENTITY_INDEX_BITS) - 1;
const ENTITY_GENERATION_MASK: u32 = (1 << ENTITY_GENERATION_BITS) - 1;
const MINIMUM_FREE_INDICES: usize = 1024;
// DO NOT CHANGE ABOVE!

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

impl Entity {
    #[inline]
    pub fn null() -> Entity {
        Entity(0)
    }

    fn new(index: u32, generation: u8) -> Entity {
        Entity(
            (index & ENTITY_INDEX_MASK)
                | ((generation as u32 & ENTITY_GENERATION_MASK) << ENTITY_INDEX_BITS),
        )
    }

    fn index(&self) -> u32 {
        self.0 & ENTITY_INDEX_MASK
    }

    fn generation(&self) -> u8 {
        ((self.0 >> ENTITY_INDEX_BITS) & ENTITY_GENERATION_MASK) as u8
    }
}

pub struct EntityManager {
    generation: Vec<u8>,
    active: Vec<bool>,
    free_indices: VecDeque<u32>,
}

impl EntityManager {
    pub fn new() -> EntityManager {
        let mut em = EntityManager {
            generation: Vec::new(),
            active: Vec::new(),
            free_indices: VecDeque::new(),
        };

        em.create_new_entity();

        em
    }

    pub fn set_entity_is_active(&mut self, entity: &Entity, active: bool) {
        if self.entity_is_alive(entity) == true {
            self.active[entity.index() as usize] = active;
        }
    }

    pub fn toggle_entity_is_active(&mut self, entity: &Entity) {
        if self.entity_is_alive(entity) == true {
            let idx = entity.index();
            self.active[idx as usize] = !self.active[idx as usize];
        }
    }

    pub fn create_new_entity(&mut self) -> Entity {
        let mut idx = 0;

        if self.free_indices.len() > MINIMUM_FREE_INDICES {
            idx = self.free_indices.pop_front().unwrap();
            self.active[idx as usize] = true;
        } else {
            self.generation.push(0);
            self.active.push(true);
            idx = self.generation.len() as u32 - 1;
        }

        Entity::new(idx, self.generation[idx as usize])
    }

    pub fn entity_is_active(&self, entity: &Entity) -> bool {
        self.active[entity.index() as usize]
    }

    pub fn entity_is_alive(&self, entity: &Entity) -> bool {
        self.generation[entity.index() as usize] == entity.generation()
    }

    pub fn destroy_entity(&mut self, entity: &Entity) {
        let idx = entity.index();
        self.generation[idx as usize] += 1;
        self.free_indices.push_back(idx);
    }
}
