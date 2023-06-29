use gamemath::Vec3;

#[derive(Clone, Copy)]
pub struct Light {
    pub position: Vec3<f32>,
    pub color: Vec3<f32>,
}
