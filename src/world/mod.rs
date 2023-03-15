use self::json_models::SceneJson;
use self::material::MaterialPtr;
use self::ray::{Ray, RayHit};
use self::shapes::{BvhNode, Cube, Shape, ShapeCollection, Sphere, ShapePtr};
use crate::algebra::transform::InversableTransform;
use crate::algebra::Vector3d;
use crate::camera::Camera;
use itertools::Itertools;
use rand::Rng;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

mod json_models;
pub mod material;
pub mod ray;
pub mod shapes;
pub mod texture;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Scene {
    shapes: Vec<ShapePtr>,
    bvh_index: BvhNode,
    camera: Camera,
    materials: HashMap<String, MaterialPtr>,
    background: Vector3d,
}

impl Scene {
    pub fn new(
        shapes: Vec<Box<dyn Shape>>,
        materials: HashMap<String, MaterialPtr>,
        camera: Camera,
        background: Vector3d,
    ) -> Self {
        let shape_arcs = shapes.into_iter().map(|shape| Arc::new(shape)).collect_vec();
        Self {
            bvh_index: BvhNode::new(&shape_arcs),
            shapes: shape_arcs,
            materials,
            camera,
            background,
        }
    }

    pub fn add_shape(&mut self, shape: Box<dyn Shape>) {
        self.shapes.push(Arc::new(shape));
    }

    pub fn closest_hit<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        self.bvh_index.ray_hit(ray, min_t, max_t)
    }

    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        let result: SceneJson = serde_json::from_str(data)?; //.map_err(|err| format!("{}", err));
        Ok(result.into())
    }

    // pub fn to_json(&self) -> String {
    //     serde_json::to_string_pretty(self.into()).unwrap()
    // }

    /// Get a reference to the scene's camera.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get a reference to the scene's background.
    pub fn background(&self, ray: &Ray) -> Vector3d {
        let t = 0.5 * (ray.direction.y + 1.0);
        (1.0 - t) * Vector3d::new(1.0, 1.0, 1.0) + t * Vector3d::new(0.5, 0.7, 1.0)
        // self.background
    }
}
