extern crate cgmath;

use cgmath::prelude::*;
use cgmath::{Point3, Vector3};
use std::f32;

use crate::materials::Material;
use crate::Ray;
use crate::shade_record::ShadeRecord;


pub trait GeometricObject {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<ShadeRecord>;
}

pub struct Sphere {
    center: Point3<f32>,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub const fn new(center: Point3<f32>, radius: f32, material: Material) -> Self {
        Sphere {
            center,
            radius,
            material,
        }
    }
}

impl GeometricObject for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<ShadeRecord> {
        let oc: Vector3<f32> = ray.origin - self.center;
        let a = ray.direction.magnitude2();
        let b = oc.dot(ray.direction);
        let c = oc.magnitude2() - self.radius * self.radius;
        let discriminant = b * b - a * c;

        let (near, far) = (
            (-b - discriminant.sqrt()) / a,
            (-b + discriminant.sqrt()) / a,
        );

        if discriminant > 0.0 {
            let option_t = match (near, far) {
                (near, _) if near > t_min && near < t_max => Some(near),
                (_, far) if far > t_min && far < t_max => Some(far),
                _ => None,
            };

            option_t.and_then(|intersect_parameter| {
                let local_hit_point = ray.point_at_parameter(intersect_parameter);
                let normal = (local_hit_point - self.center) / self.radius;

                Some(ShadeRecord {
                    intersect_parameter,
                    local_hit_point,
                    normal,
                    material: &self.material,
                })
            })
        } else {
            None
        }
    }
}

pub struct MovingSphere {
    center_start: Point3<f32>,
    center_end: Point3<f32>,
    time_start: f32,
    time_end: f32,
    radius: f32,
    material: Material,
}

impl MovingSphere {
    pub const fn new(
        center_start: Point3<f32>,
        center_end: Point3<f32>,
        time_start: f32,
        time_end: f32,
        radius: f32,
        material: Material,
    ) -> Self {
        MovingSphere {
            center_start,
            center_end,
            time_start,
            time_end,
            radius,
            material,
        }
    }

    pub fn center(&self, time: f32) -> Point3<f32> {
        self.center_start
            + ((time - self.time_start) / (self.time_end - self.time_start))
                * (self.center_end - self.center_start)
    }
}

impl GeometricObject for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<ShadeRecord> {
        let oc: Vector3<f32> = ray.origin - self.center(ray.time);
        let a = ray.direction.magnitude2();
        let b = oc.dot(ray.direction);
        let c = oc.magnitude2() - self.radius * self.radius;
        let discriminant = b * b - a * c;

        let (near, far) = (
            (-b - discriminant.sqrt()) / a,
            (-b + discriminant.sqrt()) / a,
        );

        if discriminant > 0.0 {
            let option_t = match (near, far) {
                (near, _) if near > t_min && near < t_max => Some(near),
                (_, far) if far > t_min && far < t_max => Some(far),
                _ => None,
            };

            option_t.and_then(|intersect_parameter| {
                let local_hit_point = ray.point_at_parameter(intersect_parameter);
                let normal = (local_hit_point - self.center(ray.time)) / self.radius;

                Some(ShadeRecord {
                    intersect_parameter,
                    local_hit_point,
                    normal,
                    material: &self.material,
                })
            })
        } else {
            None
        }
    }
}

pub struct Rect {
    x0: f32,
    x1: f32,
    z0: f32,
    z1: f32,
    y_height: f32,
    material: Material,
}

impl Rect {
pub const fn new(x0: f32, x1: f32, z0: f32, z1: f32, y_height: f32, material: Material) -> Self {
        Self { x0, x1, z0, z1, y_height, material }
    }
}

impl GeometricObject for Rect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<ShadeRecord> {
        let intersect_param = match (self.y_height - ray.origin.y) / ray.direction.y {
            t if t < t_min || t > t_max => None,
            t => Some(t)
        };

        intersect_param.and_then(|t| {
            match (ray.origin.x + t * ray.direction.x , ray.origin.z + t * ray.direction.z) {
                (x, z)  if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 => None,
                _ =>  
                    Some(ShadeRecord{
                        intersect_parameter: t,
                        local_hit_point: ray.point_at_parameter(t),
                        normal: Vector3::new(0.0, 0.0, 1.0),
                        material: &self.material,
                    })
            }
        })
    }
}