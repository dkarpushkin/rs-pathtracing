use crate::algebra::Vector3d;
use super::material::Material;


#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct RayHit<'a> {
    pub point: Vector3d,
    normal: Vector3d,
    pub distance: f64,
    pub is_front_face: bool,
    pub material: &'a Box<dyn Material>,
    pub u: f64,
    pub v: f64,
}

impl<'a> RayHit<'a> {
    pub fn new(
        point: Vector3d,
        normal: Vector3d,
        distance: f64,
        material: &'a Box<dyn Material>,
        ray: &Ray,
        u: f64,
        v: f64,
    ) -> Self {
        let is_front_face = &normal * &ray.direction < 0.0;
        // let normal = if is_front_face { normal } else { -normal };
        Self {
            point,
            normal: normal.normalize(),
            distance,
            is_front_face,
            material,
            u,
            v,
        }
    }

    /// Get a reference to the ray hit's normal.
    pub fn normal(&self) -> &Vector3d {
        &self.normal
    }

    /// Set the ray hit's normal.
    pub fn set_normal(&mut self, normal: Vector3d, ray: &Ray) {
        let is_front_face = &normal * &ray.direction < 0.0;
        self.normal = (if is_front_face { normal } else { -normal }).normalize();
        self.is_front_face = is_front_face;
    }
}