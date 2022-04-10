

use crate::{geometric_objects::GeometricObject, ray::Ray, shade_record::ShadeRecord};

pub struct World {
    t_min: f32,
    t_max: f32,
    objects: Vec<Box<dyn GeometricObject>>,
}

impl World {
    pub fn new() -> Self {
        World {
            objects: Vec::new(),
            t_min: 0.001,
            t_max: f32::MAX
        }
    }

    #[inline]
    pub fn add_object(&mut self, object: Box<dyn GeometricObject>) {
        self.objects.push(object);
    }

    pub fn trace(&self, ray: &Ray) -> Option<ShadeRecord> {
        let mut shade_record: Option<ShadeRecord> = None;
        let mut closest_so_far = self.t_max;

        for object in &self.objects {
            if let Some(rec) = object.hit(ray, self.t_min, closest_so_far) {
                closest_so_far = rec.intersect_parameter;
                shade_record = Some(rec);
            }
        }
        shade_record
    }
}
