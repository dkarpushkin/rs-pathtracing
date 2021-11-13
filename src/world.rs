use std::{cmp::Ordering, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::algebra::{approx_equal, Vector3d};

#[typetag::serde(tag = "type", content = "shape")]
pub trait Shape: Debug + Send + Sync {
    fn ray_intersect<'a>(&self, ray: &'a Ray, shape_box: &'a Box<dyn Shape>) -> Option<RayHit<'a>>;
}

#[derive(Debug)]
pub struct Ray {
    pub origin: Vector3d,
    pub direction: Vector3d,
}

impl Ray {
    pub fn new(origin: Vector3d, direction: Vector3d) -> Self {
        Ray {
            origin: origin,
            direction: direction,
        }
    }
}

#[derive(Debug)]
pub struct RayHit<'a> {
    pub point: Vector3d,
    pub normal: Vector3d,
    pub distance: f64,
    pub shape: &'a Box<dyn Shape>,
    pub ray: &'a Ray,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Sphere {
    center: Vector3d,
    radius: f64,
}

#[typetag::serde]
impl Shape for Sphere {
    fn ray_intersect<'a>(&self, ray: &'a Ray, shape_box: &'a Box<dyn Shape>) -> Option<RayHit<'a>> {
        let oc = &ray.origin - &self.center;
        let a = &ray.direction * &ray.direction;
        let half_b = &ray.direction * &oc;
        let c = &oc * &oc - self.radius * self.radius;

        //  решение квадратного уравнения для x - растояние от ray.origin до точек пересечения
        //  ax^2 + bx + c = 0
        let d = half_b * half_b - a * c;
        let x = if d < 0.0 {
            return None;
        } else if d == 0.0 {
            -half_b * a
        } else {
            let x1 = (-half_b + d.sqrt()) / a;
            let x2 = (-half_b - d.sqrt()) / a;
            x1.min(x2)
            // ((-half_b + d.sqrt()) / a).min((-half_b - d.sqrt()) / a)
        };

        if x < 0.0 {
            None
        } else {
            let p = &ray.origin + &ray.direction * x;
            Some(RayHit {
                normal: &p - &self.center,
                point: p,
                distance: x,
                shape: shape_box,
                ray: ray,
            })
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Plane {
    pub normal: Vector3d,
    pub p0: Vector3d,
}

#[typetag::serde]
impl Shape for Plane {
    fn ray_intersect<'a>(&self, ray: &'a Ray, shape_box: &'a Box<dyn Shape>) -> Option<RayHit<'a>> {
        let n = self.normal.normalize();
        let ln = &ray.direction.normalize() * &n;
        if approx_equal(ln, 0.0) {
            None
        } else {
            let x = ((&self.p0 - &ray.origin) * &n) / ln;
            if x > 0.0 {
                Some(RayHit {
                    normal: self.normal.clone(),
                    point: &ray.origin + x * &ray.direction,
                    distance: x,
                    shape: shape_box,
                    ray: ray,
                })
            } else {
                None
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct World {
    shapes: Vec<Box<dyn Shape>>,
}

impl World {
    pub fn closest_hit<'a>(&'a self, ray: &'a Ray) -> Option<RayHit<'a>> {
        self.shapes
            .iter()
            .filter_map(|shape| shape.ray_intersect(ray, shape))
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
}
