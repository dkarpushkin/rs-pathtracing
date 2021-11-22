use std::{cmp::Ordering, fmt::Debug};

use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::algebra::{approx_equal, Vector3d};

use self::material::Material;
use self::shapes::{Shape, Sphere};
use self::texture::SolidColor;

pub mod material;
pub mod shapes;
pub mod texture;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vector3d,
    pub direction: Vector3d,
}

impl Ray {
    pub fn new(origin: Vector3d, direction: Vector3d) -> Self {
        Ray {
            origin: origin,
            direction: direction.normalize(),
        }
    }
}

#[derive(Debug)]
pub struct RayHit<'a> {
    pub point: Vector3d,
    pub normal: Vector3d,
    pub distance: f64,
    pub is_front_face: bool,
    pub material: &'a Box<dyn Material>,
    pub ray: &'a Ray,
    pub u: f64,
    pub v: f64,
}

impl<'a> RayHit<'a> {
    fn new(
        point: Vector3d,
        normal: Vector3d,
        distance: f64,
        material: &'a Box<dyn Material>,
        ray: &'a Ray,
        u: f64,
        v: f64,
    ) -> Self {
        let is_front_face = &normal * &ray.direction < 0.0;
        let normal = if is_front_face { normal } else { -normal };
        Self {
            point,
            normal: normal.normalize(),
            distance,
            is_front_face,
            material,
            ray,
            u,
            v,
        }
    }
}

impl<'a> Eq for RayHit<'a> {}

impl<'a> PartialEq for RayHit<'a> {
    fn eq(&self, other: &Self) -> bool {
        approx_equal(self.distance, other.distance)
    }
}

impl<'a> PartialOrd for RayHit<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl<'a> Ord for RayHit<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        if approx_equal(self.distance, other.distance) {
            Ordering::Equal
        } else if self.distance < other.distance {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

fn ray_intersect<'a>(shape: &'a Box<dyn Shape>, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
    // let origin = shape.get_transform().inverse.transform_point(&ray.origin);
    // let dir = shape.get_transform().inverse.transform_vector(&ray.direction);

    shape.ray_intersect(ray, min_t, max_t)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct World {
    shapes: Vec<Box<dyn Shape>>,
}

impl World {
    pub fn closest_hit<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        self.shapes
            .iter()
            .filter_map(|shape| ray_intersect(shape, ray, min_t, max_t))
            .min_by(|a, b| {
                if approx_equal(a.distance, b.distance) {
                    Ordering::Equal
                } else if a.distance < b.distance {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
    }

    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data) //.map_err(|err| format!("{}", err))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn ad_random_spheres(&mut self, amount: u32) {
        let mut rng = rand::thread_rng();

        let spheres = self
            .shapes
            .iter()
            .filter_map(|shape| {
                let sphere = shape.as_any().downcast_ref::<Sphere>()?;
                Some((sphere.center.clone(), sphere.radius))
            })
            .collect_vec();

        for _ in 0..amount {
            let (rad, pos) = loop {
                let rad = rng.gen_range(0.2..0.7);
                let pos = Vector3d::new(
                    rng.gen_range(-10.0..10.0),
                    rad / 2.0,
                    rng.gen_range(-10.0..10.0),
                );

                if spheres.iter().any(|(c, r)| (c - &pos).length() > r + rad) {
                    break (rad, pos);
                }
            };

            let mat_choice: f64 = rng.gen();

            let mat: Box<dyn material::Material> = if mat_choice < 0.333 {
                Box::new(material::Lambertian {
                    albedo: Box::new(SolidColor {
                        color: Vector3d::random(0.0, 1.0),
                    }),
                })
            } else if mat_choice > 0.666 {
                Box::new(material::Metal {
                    albedo: Box::new(SolidColor {
                        color: Vector3d::random(0.0, 1.0),
                    }),
                    fuzz: rng.gen(),
                })
            } else {
                Box::new(material::Dielectric {
                    index_of_refraction: 1.5,
                })
            };

            let shape = shapes::Sphere {
                center: pos,
                radius: rad,
                material: mat,
            };
            self.shapes.push(Box::new(shape));
        }
    }
}
