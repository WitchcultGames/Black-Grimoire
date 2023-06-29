use fnv::FnvHashMap;
use super::super::Entity;
use gamemath::Vec3;
use gamemath::Quat;

pub struct TransformationData {
    owner: Entity,
    pub position: Vec3<f32>,
    pub scale: Vec3<f32>,
    pub pivot: Vec3<f32>,
    pub rotation: Quat,
}

pub struct TransformationSystem {
    map: FnvHashMap<Entity, usize>,
    data: Vec<TransformationData>,
}

pub struct TransformationBuilder {
    position: Option<Vec3<f32>>,
    scale: Option<Vec3<f32>>,
    pivot: Option<Vec3<f32>>,
    rotation: Option<Quat>,
}

impl TransformationBuilder {
    pub fn new() -> TransformationBuilder {
        TransformationBuilder {
            position: None,
            scale: None,
            pivot: None,
            rotation: None,
        }
    }

    pub fn at_position(mut self, position: Vec3<f32>) -> TransformationBuilder {
        self.position = Some(position);
        self
    }

    pub fn with_scale(mut self, scale: Vec3<f32>) -> TransformationBuilder {
        self.scale = Some(scale);
        self
    }

    pub fn with_pivot(mut self, pivot: Vec3<f32>) -> TransformationBuilder {
        self.pivot = Some(pivot);
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> TransformationBuilder {
        self.rotation = Some(rotation);
        self
    }

    fn build(self, owner: Entity) -> TransformationData {
        TransformationData {
            owner,
            position: match self.position {
                Some(p) => p,
                None => Vec3::new(0.0, 0.0, 0.0),
            },
            scale: match self.scale {
                Some(s) => s,
                None => Vec3::new(1.0, 1.0, 1.0),
            },
            pivot: match self.pivot {
                Some(p) => p,
                None => Vec3::new(0.0, 0.0, 0.0),
            },
            rotation: match self.rotation {
                Some(r) => r,
                None => Quat::identity(),
            },
        }
    }
}

impl TransformationSystem {
    pub fn new() -> TransformationSystem {
        TransformationSystem {
            map: FnvHashMap::with_capacity_and_hasher(1, Default::default()),
            data: Vec::new(),
        }
    }

    pub fn add_transformation_to_entity(&mut self,
                                        entity: &Entity,
                                        initial_transformation: TransformationBuilder) {
        match self.entity_has_transformation(entity) {
            true => (), //TODO: Add error logging/printing here!
            false => {
                self.data.push(initial_transformation.build(*entity));
                self.map.insert(entity.clone(), self.data.len() - 1);
            },
        }
    }

    pub fn remove_transformation_from_entity(&mut self, entity: &Entity) {
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

    pub fn entity_has_transformation(&self, entity: &Entity) -> bool {
        self.map.contains_key(entity)
    }

    pub fn get_forward_vector(&self, entity: &Entity) -> Option<Vec3<f32>> {
        match self.map.get(entity) {
            Some(index) => {
                let r = self.data[*index].rotation.extract_matrix();
                Some(r.get_forward_vector())
            },
            None => None,
        }
    }

    pub fn get_right_vector(&self, entity: &Entity) -> Option<Vec3<f32>> {
        match self.map.get(entity) {
            Some(index) => {
                let r = self.data[*index].rotation.extract_matrix();
                Some(r.get_right_vector())
            },
            None => None,
        }
    }

    pub fn get_position(&self, entity: &Entity) -> Option<Vec3<f32>> {
        match self.map.get(entity) {
            Some(index) => Some(self.data[*index].position),
            None => None,
        }
    }

    pub fn get_position_mut(&mut self, entity: &Entity) -> Option<&mut Vec3<f32>> {
        match self.map.get(entity) {
            Some(index) => Some(&mut self.data[*index].position),
            None => None,
        }
    }

    pub fn get_transformation_data(&self, entity: &Entity) -> Option<&TransformationData> {
        match self.map.get(entity) {
            Some(index) => Some(&self.data[*index]),
            None => None,
        }
    }

    pub fn rotate(&mut self, entity: &Entity, axis: Vec3<f32>, angle: f32) {
        match self.map.get(entity) {
            Some(index) => {
                self.data[*index].rotation.rotate(angle, axis);
            },
            None => (),
        }
    }

    pub fn set_rotation(&mut self, entity: &Entity, rotation: Quat) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].rotation = rotation;
                },
                None => (),
            }
        }
    }

    pub fn set_position(&mut self, entity: &Entity, position: Vec3<f32>) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].position = position;
                },
                None => (),
            }
        }
    }

    pub fn set_scale(&mut self, entity: &Entity, scale: Vec3<f32>) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].scale = scale;
                },
                None => (),
            }
        }
    }

    pub fn apply_movement(&mut self, entity: &Entity, movement: Vec3<f32>) {
        if *entity != Entity::null() {
            match self.map.get(entity) {
                Some(index) => {
                    self.data[*index].position += movement;
                },
                None => (),
            }
        }
    }
}
