use super::{
    material::{self, Material},
    shapes::{Shape, ShapeCollection, Sphere},
    texture, Scene,
};
use crate::{
    algebra::{transform::InversableTransform, Vector3d},
    camera::Camera,
};
use itertools::Itertools;
use rand::Rng;
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
        let mut shapes = scene
            .shapes
            .iter()
            .map(|shape| shape.make_shape(&materials))
            .collect_vec();
        add_random_spheres(&mut shapes);

        Scene::new(shapes, materials, scene.camera, scene.background)
    }
}

pub fn add_random_spheres(shapes: &mut Vec<Box<dyn Shape>>) {
    let mut rng = rand::thread_rng();

    // let spheres = self
    //     .world
    //     .shapes
    //     .iter()
    //     .filter_map(|shape| {
    //         let sphere = shape.as_any().downcast_ref::<Sphere>()?;
    //         Some((
    //             sphere
    //                 .transform
    //                 .direct
    //                 .transform_point(&Vector3d::new(0.0, 0.0, 0.0)),
    //             sphere
    //                 .transform
    //                 .direct
    //                 .transform_vector(&Vector3d::new(1.0, 1.0, 1.0))
    //                 .x,
    //         ))
    //     })
    //     .collect_vec();

    for (a, b) in (-11..11).cartesian_product(-11..11) {
        // let (rad, pos) = loop {
        //     let rad = rng.gen_range(0.2..0.7);
        //     let pos = Vector3d::new(
        //         rng.gen_range(-10.0..10.0),
        //         rad / 2.0,
        //         rng.gen_range(-10.0..10.0),
        //     );
        //     if spheres.iter().any(|(c, r)| (c - &pos).length() > r + rad) {
        //         break (rad, pos);
        //     }
        // };
        let center = Vector3d::new(
            a as f64 + 0.9 * rng.gen::<f64>(),
            0.2,
            b as f64 + 0.9 * rng.gen::<f64>(),
        );
        let rad = 0.2;

        if (center - Vector3d::new(4.0, 0.2, 0.0)).length() > 0.9 {
            let mat_choice: f64 = rng.gen();

            let mat: Box<dyn material::Material> = if mat_choice < 0.8 {
                let random_color = Vector3d::random(0.0, 1.0);
                Box::new(material::Lambertian {
                    albedo: Box::new(texture::SolidColor {
                        color: random_color.product(&random_color),
                    }),
                })
            } else if mat_choice < 0.95 {
                let random_color = Vector3d::random(0.0, 1.0);
                Box::new(material::Metal {
                    albedo: Box::new(texture::SolidColor {
                        color: Vector3d::new(
                            0.5 * (1.0 - random_color.x),
                            0.5 * (1.0 - random_color.y),
                            0.5 * (1.0 - random_color.z),
                        ),
                    }),
                    fuzz: 0.5 * rng.gen::<f64>(),
                })
            } else {
                Box::new(material::Dielectric {
                    index_of_refraction: 1.5,
                })
            };

            let shape = Sphere::new(
                format!(""),
                InversableTransform::new(
                    center,
                    Vector3d::new(0.0, 0.0, 0.0),
                    Vector3d::new(rad, rad, rad),
                ),
                Arc::new(mat),
            );
            shapes.push(Box::new(shape));
        }
    }
}
