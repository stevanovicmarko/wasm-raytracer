extern crate cfg_if;
extern crate wasm_bindgen;

mod geometric_objects;
mod camera;
mod materials;
mod ray;
mod scene;
mod shade_record;
mod world;

use std::mem;
use cfg_if::cfg_if;
use cgmath::{InnerSpace, Point3, vec3, Vector3, VectorSpace};
use wasm_bindgen::prelude::*;

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = Math, js_name = random)]
    pub fn random() -> f32;

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log(s: &str);

    type Crypto;
    static CRYPTO: Crypto;

    #[wasm_bindgen(method, js_name = getRandomValues)]
    fn get_random_values(this: &Crypto, values: Vec<u16>) -> Vec<u16>;
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello,{}!", name));
}

use crate::materials::{
    generate_reflect_probability, random_vec_in_unit_sphere, reflected_vector, refracted_vector,
    Material::*,
};
use crate::ray::Ray;
use crate::scene::{get_predefined_scene, get_random_scene};
use crate::world::World;

fn make_random_array(len: usize) -> Vec<f32> {
    let max_val = f32::from(u16::MAX);

    CRYPTO
        .get_random_values(vec![0_u16; len])
        .into_iter()
        .map(|x: u16| 1.0 - (2.0 * (f32::from(x) / max_val)))
        .collect::<Vec<_>>()
}

fn jittered_samples(num_samples: u8) -> Vec<(f32, f32)> {
    let n = f32::from(num_samples).sqrt() as usize;
    let mut samples = Vec::new();
    for j in 0..n {
        for k in 0..n {
            let sp = (
                (k as f32 + (1.0 - 2.0 * random())) / n as f32,
                (j as f32 + (1.0 - 2.0 * random())) / n as f32,
            );
            samples.push(sp);
        }
    }
    samples
}

const BACKGROUND_COLOR: Vector3<f32> = vec3(0.01, 0.01, 0.01);

fn generate_color_for_pixel(ray: &Ray, world: &World, depth: usize) -> Vector3<f32> {
    let shade_record = world.trace(ray);

    let pixel_color: Vector3<f32> = match (shade_record, depth < 100) {
        (_, false) => BACKGROUND_COLOR,
        (None, _) => {
            // This code adds background ambiental fake light source.
            let unit_direction = ray.direction.normalize();
            let t = (unit_direction.y + 1.0) * 0.5;
            vec3(0.1, 0.1, 0.1).lerp(BACKGROUND_COLOR, t)
        }
        // TODO: Figure out how to add time=0.0 as default param for ray class
        (Some(ref rec), true) => {
            let accumulated_color: Vector3<f32> = match &rec.material {
                Lambertian { texture } => {
                    let target = rec.local_hit_point + rec.normal + random_vec_in_unit_sphere();
                    let bounced_ray =
                        Ray::new(rec.local_hit_point, target - rec.local_hit_point, 0.0);
                    let v = generate_color_for_pixel(&bounced_ray, world, depth + 1);
                    let Point3 { x: r, y: g, z: b } = texture.value(0.0, 0.0, &rec.local_hit_point);
                    vec3(v.x * r, v.y * g, v.z * b)
                }
                Metallic { r, g, b } => {
                    let reflected = reflected_vector(&ray.direction.normalize(), &rec.normal);
                    let scattered = Ray::new(
                        rec.local_hit_point,
                        reflected + 0.5 * random_vec_in_unit_sphere(),
                        0.0,
                    );

                    if scattered.direction.dot(rec.normal) > 0.0 {
                        let u = generate_color_for_pixel(&scattered, world, depth + 1);
                        vec3(u.x * r, u.y * g, u.z * b)
                    } else {
                        generate_color_for_pixel(&scattered, world, depth + 1)
                    }
                }
                Dielectric { refractive_index } => {
                    let reflected = reflected_vector(&ray.direction, &rec.normal);
                    let ni_over_t;
                    let outward_normal;
                    let refracted;
                    let reflect_prob;
                    let mut cosine;

                    if ray.direction.dot(rec.normal) > 0.0 {
                        outward_normal = -rec.normal;
                        ni_over_t = *refractive_index;
                        cosine = ray.direction.dot(rec.normal) / ray.direction.magnitude();
                        cosine = (1.0
                            - refractive_index * refractive_index * (1.0 - cosine * cosine))
                            .sqrt();
                    } else {
                        outward_normal = rec.normal;
                        ni_over_t = 1.0 / refractive_index;
                        cosine = -ray.direction.dot(rec.normal) / ray.direction.magnitude();
                    }

                    if let Some(x) = refracted_vector(&ray.direction, &outward_normal, ni_over_t) {
                        reflect_prob = generate_reflect_probability(cosine, *refractive_index);
                        refracted = x;
                    } else {
                        reflect_prob = 1.0;
                        refracted = vec3(1.0, 1.0, 1.0);
                    };

                    let bounced_ray = if random() < reflect_prob {
                        Ray::new(rec.local_hit_point, reflected, 0.0)
                    } else {
                        Ray::new(rec.local_hit_point, refracted, 0.0)
                    };
                    generate_color_for_pixel(&bounced_ray, world, depth + 1)
                }
                DiffuseLight { texture } => {
                    let Point3 { x: r, y: g, z: b } = texture.value(
                        rec.local_hit_point.x,
                        rec.local_hit_point.y,
                        &rec.local_hit_point,
                    );
                    vec3(r, g, b)
                }
            };
            accumulated_color
        }
    };
    pixel_color
}

#[wasm_bindgen]
pub fn make_image(
    canvas_width: u16,
    canvas_height: u16,
    num_samples: u8,
    random_scene: bool,
    jittered_sampling: bool,
) -> Vec<u32> {
    let preallocate_capacity = usize::from(canvas_width) * usize::from(canvas_height);

    let samples_divider = f32::from(num_samples);

    let (camera, world) = if random_scene {
        get_random_scene(canvas_width, canvas_height, 20)
    } else {
        get_predefined_scene(canvas_width, canvas_height)
    };
    let mut pixel_color = vec3(0.0, 0.0, 0.0);
    let mut image = Vec::<u32>::with_capacity(preallocate_capacity);

    // generate precomputed displacements
    // TODO: optimize samples generation
    let samples = if jittered_sampling {
        jittered_samples(num_samples)
    } else {
        let xs = make_random_array(usize::from(num_samples));
        let ys = make_random_array(usize::from(num_samples));
        let mut vals = Vec::new();
        for i in 0..xs.len() {
            vals.push((xs[i], ys[i]));
        }
        vals
    };

    for i in 0..canvas_height {
        for j in 0..canvas_width {
            pixel_color.x = 0.0;
            pixel_color.y = 0.0;
            pixel_color.z = 0.0;

            for sample in &samples {
                let dx = (f32::from(j) + sample.0) / f32::from(canvas_width);
                let dy = (f32::from(i) + sample.1) / f32::from(canvas_height);

                let direction = camera.get_ray(dx, dy);
                pixel_color += generate_color_for_pixel(&direction, &world, 0);
            }
            pixel_color /= samples_divider;

            let Vector3 { x: r, y: g, z: b } = pixel_color;

            let pixel = unsafe {
                mem::transmute::<[u8; 4], u32>([
                    (r.sqrt() * 255.99) as u8,
                    (g.sqrt() * 255.99) as u8,
                    (b.sqrt() * 255.99) as u8,
                    255,
                ])
            };
            image.push(pixel);
        }
    }
    image
}
