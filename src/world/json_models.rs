use super::{material::Material, shapes::ShapeCollection, Scene};
use crate::{algebra::Vector3d, camera::Camera};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, sync::Arc};

#[typetag::serde(tag = "type")]
pub trait ShapeJson: Debug {
    fn make_shape(
        &self,
        materials: &HashMap<String, Arc<Box<dyn Material>>>,
    ) -> Box<dyn super::Shape>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneJson {
    camera: Camera,
    shapes: Vec<Box<dyn ShapeJson>>,
    materials: HashMap<String, Box<dyn Material>>,
    background: Vector3d,
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
            background: scene.background,
        }
    }
}
