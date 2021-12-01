use std::fmt::Debug;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::algebra::Vector3d;

use super::{Ray, RayHit, texture::Texture};

pub struct Scatter {
    pub ray: Ray,
    pub attenuation: Vector3d,
}

impl Scatter {
    fn new(ray: Ray, attenuation: Vector3d) -> Self {
        Self { ray, attenuation }
    }
}

#[typetag::serde(tag = "type")]
pub trait Material: Debug + Send + Sync {
    fn scatter(&self, _ray: &Ray, _ray_hit: &RayHit) -> Option<Scatter> {
        None
    }

    fn emitted(&self, _u: f64, _v: f64, _p: &Vector3d) -> Vector3d {
        Vector3d::new(0.0, 0.0, 0.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lambertian {
    pub albedo: Box<dyn Texture>,
}

#[typetag::serde]
impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, ray_hit: &RayHit) -> Option<Scatter> {
        let mut direction = ray_hit.normal() + Vector3d::random_unit();
        // let direction = &ray_hit.normal + Vector3d::random_in_hemisphere(&ray_hit.normal);
        if direction.is_zero() {
            direction = ray_hit.normal().clone()
        }

        Some(Scatter::new(
            Ray::new(ray_hit.point.clone(), direction),
            self.albedo.value(ray_hit.u, ray_hit.v, &ray_hit.point),
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metal {
    pub albedo: Box<dyn Texture>,
    pub fuzz: f64,
}

#[typetag::serde]
impl Material for Metal {
    fn scatter(&self, ray: &Ray, ray_hit: &RayHit) -> Option<Scatter> {
        let reflected = ray.direction.reflect(ray_hit.normal());
        let direction = if self.fuzz == 0.0 {
            reflected
        } else {
            reflected + self.fuzz * Vector3d::random_in_unit_sphere()
        };
        Some(Scatter::new(
            Ray::new(ray_hit.point.clone(), direction),
            self.albedo.value(ray_hit.u, ray_hit.v, &ray_hit.point),
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dielectric {
    pub index_of_refraction: f64,
}

impl Dielectric {
    fn reflectance(cosine: f64, ref_index: f64) -> f64 {
        let r0 = (1.0 - ref_index) / (1.0 + ref_index);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

#[typetag::serde]
impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, ray_hit: &RayHit) -> Option<Scatter> {
        let refract_ratio = if ray_hit.is_front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };

        let cos_theta = -&ray.direction * ray_hit.normal();
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let direction = if refract_ratio * sin_theta > 1.0
            || Dielectric::reflectance(cos_theta, refract_ratio) > rand::thread_rng().gen()
        {
            ray.direction.reflect(ray_hit.normal())
        } else {
            ray.direction.refract(ray_hit.normal(), refract_ratio)
        };

        Some(Scatter::new(
            Ray::new(ray_hit.point.clone(), direction),
            Vector3d::new(1.0, 1.0, 1.0),
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiffuseLight {
    pub emit: Box<dyn Texture>,
}

#[typetag::serde]
impl Material for DiffuseLight {
    fn emitted(&self, u: f64, v: f64, p: &Vector3d) -> Vector3d {
        self.emit.value(u, v, p)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmptyMaterial;

#[typetag::serde]
impl Material for EmptyMaterial {}
