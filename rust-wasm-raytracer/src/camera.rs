use cgmath::prelude::*;
use cgmath::{vec3, Point3, Vector3};
use std::f32;
use rand::random;
use crate::Ray;

fn random_vec_in_disc() -> Vector3<f32> {
    let r = random::<f32>().sqrt();
    let theta = 2.0 * f32::consts::PI * random::<f32>();
    let x = r * theta.cos();
    let y = r * theta.sin();

    random::<f32>() * vec3(x, y, 0.0)
}

pub struct Camera {
    origin: Point3<f32>,
    horizontal: Vector3<f32>,
    vertical: Vector3<f32>,
    top_left_corner: Vector3<f32>,
    u: Vector3<f32>,
    v: Vector3<f32>,
    lens_radius: f32,
    time_start: f32,
    time_end: f32,
}

impl Camera {
    pub fn new(
        look_from: &Point3<f32>,
        look_at: &Point3<f32>,
        up: &Vector3<f32>,
        v_fov: f32,
        aspect: f32,
        aperture: f32,
        focus_distance: f32,
        time_start: f32,
        time_end: f32,
    ) -> Self {
        let lens_radius = aperture / 2.0;
        let theta = v_fov * f32::consts::PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let half_width = aspect * half_height;
        let origin = *look_from;
        let w = (look_from - look_at).normalize();
        let u = up.cross(w).normalize();
        let v = w.cross(u);

        let p: Point3<f32> = Point3::from(<Vector3<f32> as Into<(f32, f32, f32)>>::into(
            u * half_width * focus_distance - v * half_height * focus_distance,
        ));

        let top_left_corner: Vector3<f32> = origin - p - (w * focus_distance);
        let horizontal = u * half_width * (2.0 * focus_distance);
        let vertical = v * half_height * (2.0 * focus_distance);

        Camera {
            origin,
            horizontal,
            vertical,
            top_left_corner,
            u,
            v,
            lens_radius,
            time_start,
            time_end,
        }
    }

    pub fn get_ray(&self, u_coord: f32, v_coord: f32) -> Ray {
        let rd = random_vec_in_disc() * self.lens_radius;
        let offset = (self.u * rd.x) + (self.v * rd.y);
        let (x, y, z) = ((self.horizontal * u_coord) + (self.vertical * - v_coord) - offset).into();
        let time = self.time_start + random::<f32>() * (self.time_end - self.time_start);
        Ray::new(
            self.origin + offset,
            self.top_left_corner + (Point3::new(x, y, z) - self.origin),
            time,
        )
    }
}
