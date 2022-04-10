use cgmath::{Point3, Vector3};

use crate::materials::Material;

pub struct ShadeRecord<'a> {
    pub normal: Vector3<f32>,
    pub local_hit_point: Point3<f32>,
    pub material: &'a Material,
    pub intersect_parameter: f32,
}
