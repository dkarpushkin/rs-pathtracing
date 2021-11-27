use self::json_models::SceneJson;
use self::material::Material;
use self::ray::{Ray, RayHit};
use self::shapes::{Shape, ShapeCollection};
use crate::camera::Camera;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

pub mod material;
pub mod ray;
pub mod shapes;
pub mod texture;

#[derive(Debug)]
pub struct Scene {
    pub world: ShapeCollection,
    pub camera: Camera,
    pub materials: HashMap<String, Arc<Box<dyn Material>>>,
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

mod json_models {
    use std::{collections::HashMap, sync::Arc};

    use crate::camera::Camera;

    use super::{
        material::Material, shapes::json_models::ShapeJson, shapes::ShapeCollection, Scene,
    };
    use itertools::Itertools;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SceneJson {
        camera: Camera,
        shapes: Vec<Box<dyn ShapeJson>>,
        materials: HashMap<String, Box<dyn Material>>,
    }

    impl From<SceneJson> for Scene {
        fn from(scene: SceneJson) -> Self {
            let materials: HashMap<String, Arc<Box<dyn Material>>> = HashMap::from_iter(
                scene
                    .materials
                    .into_iter()
                    .map(|(key, mat)| (key, Arc::new(mat))),
            );
            let shapes = scene
                .shapes
                .iter()
                .map(|shape| shape.make_shape(&materials))
                .collect_vec();
            let world = ShapeCollection::new("World", shapes);

            Scene {
                world,
                camera: scene.camera,
                materials: materials,
            }
        }
    }
}
