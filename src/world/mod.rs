use self::material::Material;
use self::ray::{Ray, RayHit};
use self::shapes::{Shape, ShapeCollection};
use crate::camera::Camera;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod material;
pub mod ray;
pub mod shapes;
pub mod texture;

#[derive(Serialize, Deserialize, Debug)]
// #[serde(from = "json_models::SceneJson")]
pub struct Scene {
    pub world: ShapeCollection,
    pub camera: Camera,
    pub materials: Vec<Box<dyn Material>>,
}

impl Scene {
    pub fn closest_hit<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        self.world.ray_intersect(ray, min_t, max_t)
    }

    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data) //.map_err(|err| format!("{}", err))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

// mod json_models {
//     use super::{
//         material::Material, shapes::json_models::ShapeJson, shapes::ShapeCollection, Scene,
//     };
//     use serde::{Deserialize, Serialize};

//     #[derive(Serialize, Deserialize, Debug)]
//     pub struct SceneJson {
//         shapes: Vec<Box<dyn ShapeJson>>,
//         materials: Vec<Box<dyn Material>>,
//     }

//     impl From<SceneJson> for Scene {
//         fn from(scene: SceneJson) -> Self {
//             let world = ShapeCollection::new();
//         }
//     }
// }
