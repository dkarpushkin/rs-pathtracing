use self::json_models::SceneJson;
use self::material::Material;
use self::ray::{Ray, RayHit};
use self::shapes::{BvhNode, Cube, Shape, ShapeCollection, Sphere};
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
    world: Box<dyn Shape>,
    camera: Camera,
    materials: HashMap<String, Arc<Box<dyn Material>>>,
    background: Vector3d,
}

impl Scene {
    pub fn new(
        shapes: Vec<Box<dyn Shape>>,
        materials: HashMap<String, Arc<Box<dyn Material>>>,
        camera: Camera,
        background: Vector3d,
    ) -> Self {
        Self {
            world: Box::new(BvhNode::new(shapes)) as Box<dyn Shape>,
            materials,
            camera,
            background,
        }
    }

    pub fn closest_hit<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        self.world.ray_hit(ray, min_t, max_t)
    }

    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        let result: SceneJson = serde_json::from_str(data)?; //.map_err(|err| format!("{}", err));
        Ok(result.into())
    }

    // pub fn to_json(&self) -> String {
    //     serde_json::to_string_pretty(self.into()).unwrap()
    // }

    pub fn generate_cubes(&mut self, number: u32) {
        let mut rng = rand::thread_rng();

        let materials = (0..(number * 2))
            .map(|_| {
                let mat_choice: f64 = rng.gen();

                let mat: Box<dyn material::Material> = if mat_choice < 0.333 {
                    Box::new(material::Lambertian {
                        albedo: Box::new(texture::SolidColor {
                            color: Vector3d::random(0.0, 1.0),
                        }),
                    })
                } else if mat_choice > 0.666 {
                    Box::new(material::Metal {
                        albedo: Box::new(texture::SolidColor {
                            color: Vector3d::random(0.0, 1.0),
                        }),
                        fuzz: rng.gen(),
                    })
                } else {
                    Box::new(material::Dielectric {
                        index_of_refraction: 1.5,
                    })
                };

                Arc::new(mat)
            })
            .collect_vec();

        let cube_width = 10.0;
        let shapes: Vec<Box<dyn Shape>> = (0..number)
            .cartesian_product(0..number)
            .map(|(x, z)| {
                let mat_choice = rng.gen_range(0..(number * 2));

                let shape = Cube::new(
                    format!("Cube_{}_{}", x, z),
                    InversableTransform::new(
                        Vector3d::new(x as f64 * cube_width, 0.0, z as f64 * cube_width),
                        Vector3d::zero(),
                        Vector3d::new(cube_width / 2.0, rng.gen_range(2.5..5.0), cube_width / 2.0),
                    ),
                    materials[mat_choice as usize].clone(),
                );

                Box::new(shape) as Box<dyn Shape>
            })
            .collect_vec();

        self.world = Box::new(ShapeCollection::new("World", shapes));
    }

    pub fn add_random_spheres(&mut self) {
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

            if (&center - Vector3d::new(4.0, 0.2, 0.0)).length() > 0.9 {
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
                // self.world.shapes.push(Box::new(shape));
            }
        }
    }

    /// Get a reference to the scene's camera.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}
