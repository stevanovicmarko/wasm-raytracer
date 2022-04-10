use arr_macro::arr;
use cgmath::prelude::*;
use cgmath::{vec3, Point3, Vector3};
use lazy_static::lazy_static;
use std::f32;
use rand::random;
use rand::seq::SliceRandom;
use rand::thread_rng;


#[inline]
pub fn reflected_vector(v: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    v - n * 2.0 * v.dot(*n)
}

#[inline]
pub fn generate_reflect_probability(cosine: f32, refractive_index: f32) -> f32 {
    let mut r0 = (1.0 - refractive_index) / (1.0 + refractive_index);
    r0 = r0 * r0;
    r0 + ((1.0 - r0) * ((1.0 - cosine).powf(5.0)))
}

pub fn refracted_vector(
    v: &Vector3<f32>,
    n: &Vector3<f32>,
    ni_over_nt: f32,
) -> Option<Vector3<f32>> {
    let uv = v.normalize();
    let dt = uv.dot(*n);
    let discriminant = 1.0 - (ni_over_nt * ni_over_nt) * (1.0 - (dt * dt));
    if discriminant > 0.0 {
        let refracted = ni_over_nt * (uv - n * dt) - (n * (discriminant.sqrt()));
        Some(refracted)
    } else {
        None
    }
}

pub fn random_vec_in_unit_sphere() -> Vector3<f32> {
    let z = 1.0 - (2.0 * random::<f32>());
    let r = (1.0 - (z * z)).sqrt();
    let theta = 2.0 * f32::consts::PI * random::<f32>();
    let x = r * theta.cos();
    let y = r * theta.sin();

    random::<f32>() * vec3(x, y, z)
}

pub enum Texture {
    Constant {
        color: Point3<f32>,
    },
    Checkerboard {
        left: Box<Texture>,
        right: Box<Texture>,
    },
    Noise,
}

pub struct Perlin {
    pub scale_factor: f32,
    pub random_vecs: [Vector3<f32>; 256],
    pub random_x_direction: [i32; 256],
    pub random_y_direction: [i32; 256],
    pub random_z_direction: [i32; 256],
}

// TODO: Refactor perlin implementation
impl Perlin {
    pub fn new() -> Self {
        Perlin {
            scale_factor: 5.0,
            random_vecs: Perlin::perlin_generate(),
            random_x_direction: Perlin::generate_perm(),
            random_y_direction: Perlin::generate_perm(),
            random_z_direction: Perlin::generate_perm(),
        }
    }

    #[inline]
    pub fn perlin_generate() -> [Vector3<f32>; 256] {
        arr![vec3(-1.0 + 2.0 * random::<f32>(), -1.0 + 2.0 * random::<f32>(), -1.0 + 2.0 * random::<f32>()).normalize(); 256]
    }

    pub fn generate_perm() -> [i32; 256] {
        let mut i = -1_i32;
        let mut shuffled_array = arr![{ i += 1; i}; 256];
        let mut rng = thread_rng();
        shuffled_array.shuffle(&mut rng);
        shuffled_array
    }

    pub fn generate_noise(point: &Point3<f32>) -> f32 {
        let scaled_point = point * 1.0;
        let u = scaled_point.x - scaled_point.x.floor();
        let v = scaled_point.y - scaled_point.y.floor();
        let w = scaled_point.z - scaled_point.z.floor();

        let i = scaled_point.x.floor() as i32;
        let j = scaled_point.y.floor() as i32;
        let k = scaled_point.z.floor() as i32;

        let mut c: [[[Vector3<f32>; 2]; 2]; 2] = [
            [
                [vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0)],
                [vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0)],
            ],
            [
                [vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0)],
                [vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0)],
            ],
        ];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let x_idx = ((i + di) & 255) as usize;
                    let y_idx = ((j + dj) & 255) as usize;
                    let z_idx = ((k + dk) & 255) as usize;

                    let rand_x = PERLIN_STATIC_REF.random_x_direction[x_idx];
                    let rand_y = PERLIN_STATIC_REF.random_y_direction[y_idx];
                    let rand_z = PERLIN_STATIC_REF.random_z_direction[z_idx];

                    let index = (rand_x ^ rand_y ^ rand_z) as usize;

                    c[di as usize][dj as usize][dk as usize] = PERLIN_STATIC_REF.random_vecs[index];
                }
            }
        }

        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut acc = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_v = vec3(u - i as f32, v - j as f32, w - k as f32);
                    acc += (i as f32 * uu + (1.0 - i as f32) * (1.0 - uu))
                        * (j as f32 * vv + (1.0 - j as f32) * (1.0 - vv))
                        * (k as f32 * ww + (1.0 - k as f32) * (1.0 - ww))
                        * weight_v.dot(c[i as usize][j as usize][k as usize]);
                }
            }
        }

        acc
    }
}

lazy_static! {
    static ref PERLIN_STATIC_REF: Perlin = Perlin::new();
}

impl Texture {
    pub fn value(&self, x: f32, y: f32, point: &Point3<f32>) -> Point3<f32> {
        match self {
            Texture::Constant { color } => *color,
            Texture::Checkerboard { left, right } => {
                let sines =
                    f32::sin(10.0 * point.x) * f32::sin(10.0 * point.y) * f32::sin(10.0 * point.z);

                if sines < 0.0 {
                    left.value(x, y, point)
                } else {
                    right.value(x, y, point)
                }
            }
            Texture::Noise => {
                let mut acc = 0.0;
                let mut temp_p = *point;
                let mut weight = 1.0;
                for _i in 0..7 {
                    acc += weight * Perlin::generate_noise(&temp_p);
                    weight *= 0.5;
                    temp_p *= 2.0;
                }

                Point3::new(1.0, 1.0, 1.0)
                    * 0.5
                    * (1.0 + f32::sin(PERLIN_STATIC_REF.scale_factor * point.z + 10.0 * acc))
            }
        }
    }
}

pub enum Material {
    Lambertian { texture: Texture },
    Metallic { r: f32, g: f32, b: f32 },
    Dielectric { refractive_index: f32 },
    DiffuseLight { texture: Texture },
}
