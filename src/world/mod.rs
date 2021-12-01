use self::json_models::SceneJson;
use self::material::Material;
use self::ray::{Ray, RayHit};
use self::shapes::{Shape, ShapeCollection};
use crate::algebra::Vector3d;
use crate::camera::Camera;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

pub mod material;
pub mod ray;
pub mod shapes;
pub mod texture;
mod json_models;

#[derive(Debug)]
pub struct Scene {
    pub world: ShapeCollection,
    pub camera: Camera,
    pub materials: HashMap<String, Arc<Box<dyn Material>>>,
    pub background: Vector3d,
}

impl Scene {
    pub fn closest_hit<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        self.world.ray_intersect(ray, min_t, max_t)
    }

    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        let result: SceneJson = serde_json::from_str(data)?; //.map_err(|err| format!("{}", err));
        Ok(result.into())
    }

    // pub fn to_json(&self) -> String {
    //     serde_json::to_string_pretty(self.into()).unwrap()
    // }
}
