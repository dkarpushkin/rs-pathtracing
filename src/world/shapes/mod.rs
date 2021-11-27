use super::{
    material::{self, Material},
    texture, Ray, RayHit,
};
use crate::algebra::{
    approx_equal, equation::solve_quantic_equation, transform::InversableTransform, Vector3d,
};
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{any::Any, cmp::Ordering, f64::consts::PI, fmt::Debug, str::FromStr};

pub mod brute_forced;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AABB {
    min_p: Vector3d,
    max_p: Vector3d,
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min_p: Vector3d {
                x: -1.0,
                y: -1.0,
                z: -1.0,
            },
            max_p: Vector3d {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }
    }
}

impl AABB {
    fn ray_hit(&self, ray: &Ray, min_t: f64, max_t: f64) -> bool {
        let t_lower = (&self.min_p - &ray.origin).divide(&ray.direction);
        let t_upper = (&self.max_p - &ray.origin).divide(&ray.direction);

        let t_mins = t_lower.min(&t_upper);
        let t_maxes = t_lower.max(&t_upper);

        let t_box_min = t_mins.max_component().max(min_t);
        let t_box_max = t_maxes.min_component().min(max_t);

        t_box_min <= t_box_max
    }
}

#[typetag::serde(tag = "type")]
pub trait Shape: Debug + Send + Sync {
    fn ray_hit_bounded<'a>(&'a self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        if let Some(bound) = self.get_bounding_box() {
            if bound.ray_hit(ray, min_t, max_t) {
                self.ray_intersect(ray, min_t, max_t)
            } else {
                None
            }
        } else {
            self.ray_intersect(ray, min_t, max_t)
        }
    }

    fn ray_hit<'a>(&'a self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        if let Some(transform) = self.get_transform() {
            let mut ret =
                self.ray_hit_bounded(&transform.inverse_transform_ray(&ray), min_t, max_t)?;

            ret.point = transform.direct.transform_point(&ret.point);
            ret.set_normal(transform.inverse.transform_normal(&ret.normal()), ray);

            Some(ret)
        } else {
            self.ray_hit_bounded(ray, min_t, max_t)
        }
    }

    fn ray_intersect<'a>(&'a self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>>;

    fn get_bounding_box(&self) -> Option<&AABB> {
        None
    }

    fn get_transform(&self) -> Option<&InversableTransform> {
        None
    }

    fn as_any(&self) -> &dyn Any;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sphere {
    name: String,
    transform: InversableTransform,
    material: Box<dyn Material>,
    #[serde(default)]
    aabb: AABB,
}

#[typetag::serde]
impl Shape for Sphere {
    fn ray_intersect<'a>(&'a self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let origin = &ray.origin;
        let dir = &ray.direction;

        let a = dir * dir;
        let half_b = dir * origin;
        let c = origin * origin - 1.0;

        //  решение квадратного уравнения для x - растояние от ray.origin до точек пересечения
        //  ax^2 + bx + c = 0
        let d = half_b * half_b - a * c;
        let x = if d < 0.0 {
            return None;
        } else if d == 0.0 {
            -half_b * a
        } else {
            let x = (-half_b - d.sqrt()) / a;
            if x < min_t || x > max_t {
                let x = (-half_b + d.sqrt()) / a;
                if x < min_t || x > max_t {
                    return None;
                }
                x
            } else {
                x
            }
        };

        let p = origin + dir * x;
        let normal = p.clone();

        // let theta = (p.z).acos();
        // let phi = (p.y).atan2(p.x) + PI;
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;
        Some(RayHit::new(
            p,
            normal,
            x,
            &self.material,
            ray,
            phi / (2.0 * PI),
            theta / PI,
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_transform(&self) -> Option<&InversableTransform> {
        Some(&self.transform)
    }

    fn get_bounding_box(&self) -> Option<&AABB> {
        Some(&self.aabb)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Torus {
    name: String,
    radius: f64,
    tube_radius: f64,
    transform: InversableTransform,
    material: Box<dyn Material>,
}

#[typetag::serde]
impl Shape for Torus {
    fn ray_intersect<'a>(&'a self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let origin = &ray.origin;
        let dir = &ray.direction;

        let t = 4.0 * self.radius * self.radius;
        let g = t * (dir.x * dir.x + dir.y * dir.y);
        let h = 2.0 * t * (origin.x * dir.x + origin.y * dir.y);
        let i = t * (origin.x * origin.x + origin.y * origin.y);
        let j = dir.squared_length();
        let k = 2.0 * (origin * dir);
        let l = origin.squared_length() + self.radius * self.radius
            - self.tube_radius * self.tube_radius;

        let a = j * j;
        let b = 2.0 * j * k;
        let c = 2.0 * j * l + k * k - g;
        let d = 2.0 * k * l - h;
        let e = l * l - i;

        let mut min_root = f64::INFINITY;
        let roots = solve_quantic_equation(a.into(), b.into(), c.into(), d.into(), e.into());
        for root in roots {
            if approx_equal(root.im, 0.0) && root.re < min_root {
                // if root.im == 0.0 && root.re < min_root {
                min_root = root.re;
            }
        }

        if min_root.is_infinite() || min_root < min_t || min_root > max_t {
            None
        } else {
            let p = origin + min_root * dir;
            let normal = &p - Vector3d::new(p.x, p.y, 0.0).normalize() * self.radius;
            let theta = (p.z / self.tube_radius).asin();
            let phi = (p.z / (self.radius + self.tube_radius * theta.cos())).acos() + PI;

            Some(RayHit::new(
                p,
                normal,
                min_root,
                &self.material,
                ray,
                phi / (2.0 * PI),
                theta / PI,
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_transform(&self) -> Option<&InversableTransform> {
        Some(&self.transform)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Tooth {
    name: String,
    transform: InversableTransform,
    material: Box<dyn Material>,
}

#[typetag::serde]
impl Shape for Tooth {
    fn ray_intersect<'a>(&'a self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        // let o = self.transform.inverse.transform_point(&ray.origin);
        // let dir = self.transform.inverse.transform_vector(&ray.direction);
        let o = &ray.origin;
        let dir = &ray.direction;

        let dir2 = dir.powi(2);
        let o2 = o.powi(2);

        let a = &dir2 * &dir2;
        let b = 4.0 * (dir.powi(3) * o);
        let c = 6.0 * (dir2 * &o2) - o.squared_length();
        let d = 4.0 * (dir * o.powi(3)) - 2.0 * (o * dir);
        let e = &o2 * &o2 - o * o;

        let mut min_root = f64::INFINITY;
        let roots = solve_quantic_equation(a.into(), b.into(), c.into(), d.into(), e.into());
        for root in roots {
            if approx_equal(root.im, 0.0) && root.re < min_root {
                // if root.im == 0.0 && root.re < min_root {
                min_root = root.re;
            }
        }

        if min_root.is_infinite() || min_root < min_t || min_root > max_t {
            None
        } else {
            let p = o + min_root * dir;

            let normal = Vector3d::new(
                4.0 * p.x.powi(3) - 2.0 * p.x,
                4.0 * p.y.powi(3) - 2.0 * p.y,
                4.0 * p.z.powi(3) - 2.0 * p.z,
            );

            Some(RayHit::new(
                p,
                normal,
                min_root,
                &self.material,
                ray,
                0.0,
                0.0,
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_transform(&self) -> Option<&InversableTransform> {
        Some(&self.transform)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShapeCollection {
    name: String,
    shapes: Vec<Box<dyn Shape>>,
}

#[typetag::serde]
impl Shape for ShapeCollection {
    fn ray_intersect<'a>(&'a self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        self.shapes
            .iter()
            .filter_map(|shape| shape.ray_hit(ray, min_t, max_t))
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ShapeCollection {
    pub fn new(name: String, shapes: Vec<Box<dyn Shape>>) -> Self {
        Self { name, shapes }
    }

    pub fn ad_random_spheres(&mut self, amount: u32) {
        let mut rng = rand::thread_rng();

        let spheres = self
            .shapes
            .iter()
            .filter_map(|shape| {
                let sphere = shape.as_any().downcast_ref::<Sphere>()?;
                Some((
                    sphere
                        .transform
                        .direct
                        .transform_point(&Vector3d::new(0.0, 0.0, 0.0)),
                    sphere
                        .transform
                        .direct
                        .transform_vector(&Vector3d::new(1.0, 1.0, 1.0))
                        .x,
                ))
            })
            .collect_vec();

        for _ in 0..amount {
            let (rad, pos) = loop {
                let rad = rng.gen_range(0.2..0.7);
                let pos = Vector3d::new(
                    rng.gen_range(-10.0..10.0),
                    rad / 2.0,
                    rng.gen_range(-10.0..10.0),
                );

                if spheres.iter().any(|(c, r)| (c - &pos).length() > r + rad) {
                    break (rad, pos);
                }
            };

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

            let shape = Sphere {
                material: mat,
                transform: InversableTransform::new(
                    pos,
                    Vector3d::new(0.0, 0.0, 0.0),
                    Vector3d::new(rad, rad, rad),
                ),
                aabb: AABB::default(),
                name: String::from_str("Sphere1").unwrap(),
            };
            self.shapes.push(Box::new(shape));
        }
    }
}

// pub mod json_models {
//     use super::{material::Material, Shape, Sphere, AABB};
//     use crate::algebra::transform::InversableTransform;
//     use serde::{Deserialize, Serialize};
//     use std::{collections::HashMap, fmt::Debug};

//     pub trait ShapeJson: Debug + Serialize {
//         fn make_shape(&self, materials: HashMap<String, Box<dyn Material>>) -> Box<dyn Shape>;
//     }

//     #[derive(Serialize, Deserialize, Debug)]
//     struct SphereJson {
//         transform: InversableTransform,
//         name: String,
//         material_name: String,
//     }

//     impl ShapeJson for SphereJson {
//         fn make_shape(&self, materials: HashMap<String, Box<dyn Material>>) -> Box<dyn Shape> {
//             let aabb = AABB::default();
//             Box::new(Sphere {
//                 name: self.name.clone(),
//                 transform: self.transform.clone(),
//                 material: materials[&self.material_name],
//                 aabb: aabb,
//             })
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        algebra::{transform::InversableTransform, Vector3d},
        world::{material::EmptyMaterial, Ray, Shape},
    };

    use super::Torus;

    #[test]
    fn test_torus() {
        let mat = EmptyMaterial;

        let ray = Ray::new(
            Vector3d::new(0.0, 0.0, -10.0),
            Vector3d::new(
                0.42233513247717097,
                0.26611434880691537,
                -0.86649650272494549,
            ),
        );

        let tor = Torus {
            name: String::from_str("Torus").unwrap(),
            radius: 0.5,
            tube_radius: 0.1,
            transform: InversableTransform::new(
                Vector3d::new(0.0, 0.0, 0.0),
                Vector3d::new(0.0, 0.0, 0.0),
                Vector3d::new(1.0, 1.0, 1.0),
            ),
            material: Box::new(mat),
        };
        println!("Torus: {:?}", tor);

        let hit = tor.ray_intersect(&ray, 0.001, f64::INFINITY);

        println!("{:?}", hit);
    }
}
