use cgmath::{Point3, Vector3};
use std::f32;

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub time: f32,
}

impl Ray {
    pub const fn new(origin: Point3<f32>, direction: Vector3<f32>, time: f32) -> Self {
        Ray {
            origin,
            direction,
            time,
        }
    }

    #[inline]
    pub fn point_at_parameter(&self, t: f32) -> Point3<f32> {
        self.origin + (self.direction * t)
    }
}
