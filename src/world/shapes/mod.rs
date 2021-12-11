use itertools::Itertools;
use rand::Rng;

use super::{
    material::{self, Material},
    Ray, RayHit,
};
use crate::algebra::{
    approx_equal,
    equation::solve_quantic_equation,
    transform::{InversableTransform, Transform},
    Vector3d,
};
use std::{any::Any, f64::consts::PI, fmt::Debug, ops::Index, sync::Arc};

pub mod ray_marching;

#[derive(Clone, Debug)]
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

impl Index<usize> for AABB {
    type Output = Vector3d;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.min_p,
            1 => &self.max_p,
            _ => panic!("Wrong index in AABB"),
        }
    }
}

#[allow(dead_code)]
impl AABB {
    fn maximum() -> Self {
        AABB {
            min_p: Vector3d::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            max_p: Vector3d::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
        }
    }

    fn minimum() -> Self {
        AABB {
            min_p: Vector3d::new(0.0, 0.0, 0.0),
            max_p: Vector3d::new(0.0, 0.0, 0.0),
        }
    }

    fn ray_hit(&self, ray: &Ray, min_t: f64, max_t: f64) -> bool {
        let t_lower = (&self.min_p - &ray.origin).divide(&ray.direction);
        let t_upper = (&self.max_p - &ray.origin).divide(&ray.direction);

        let t_mins = t_lower.min(&t_upper);
        let t_maxes = t_lower.max(&t_upper);

        let t_box_min = t_mins.max_component().max(min_t);
        let t_box_max = t_maxes.min_component().min(max_t);

        t_box_min <= t_box_max
    }

    fn max(&self, other: &AABB) -> AABB {
        AABB {
            min_p: self.min_p.min(&other.min_p),
            max_p: self.max_p.max(&other.max_p),
        }
    }

    fn enlarge(&mut self, other: &AABB) {
        self.min_p = self.min_p.min(&other.min_p);
        self.max_p = self.max_p.max(&other.max_p);
    }

    fn transform(&self, transform: &Transform) -> AABB {
        let mut points = Vec::with_capacity(8);
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    points.push(Vector3d::new(self[i].x, self[j].y, self[k].z));
                }
            }
        }
        let (min_p, max_p) = points.iter().map(|p| transform.transform_point(p)).fold(
            (Vector3d::infinity(), Vector3d::neg_infinity()),
            |(min_p, max_p), p| (min_p.min(&p), max_p.max(&p)),
        );

        AABB { min_p, max_p }
    }
}

pub trait Shape: Debug + Send + Sync {
    fn ray_hit_transformed(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
        if let Some(transform) = self.get_transform() {
            let mut ret =
                self.ray_intersect(&transform.inverse_transform_ray(&ray), min_t, max_t)?;

            ret.point = transform.direct.transform_point(&ret.point);
            ret.set_normal(transform.inverse.transform_normal(&ret.normal()), ray);

            Some(ret)
        } else {
            self.ray_intersect(ray, min_t, max_t)
        }
    }

    fn ray_hit(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
        // if let Some(bound) = self.get_bounding_box() {
        //     if bound.ray_hit(ray, min_t, max_t) {
        //         self.ray_hit_transformed(ray, min_t, max_t)
        //     } else {
        //         None
        //     }
        // } else {
        //     self.ray_hit_transformed(ray, min_t, max_t)
        // }
        self.ray_hit_transformed(ray, min_t, max_t)
    }

    fn ray_intersect(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit>;

    fn get_bounding_box(&self) -> AABB;

    fn get_transform(&self) -> Option<&InversableTransform> {
        None
    }

    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
struct Rectangle {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    transform: InversableTransform,
    material: Arc<Box<dyn Material>>,
}

impl Rectangle {
    fn new(
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        transform: InversableTransform,
        material: Arc<Box<dyn Material>>,
    ) -> Self {
        Self {
            x0,
            y0,
            x1,
            y1,
            transform,
            material,
        }
    }
}

impl Shape for Rectangle {
    fn ray_intersect(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
        let t = -ray.origin.z / ray.direction.z;
        if t < min_t || t > max_t {
            None
        } else {
            let p = &ray.origin + &ray.direction * t;

            if p.x < self.x0 || p.x > self.x1 || p.y < self.y0 || p.y > self.y1 {
                None
            } else {
                let u = (p.x - self.x0) / (self.x1 - self.x0);
                let v = (p.y - self.y0) / (self.y1 - self.y0);
                Some(RayHit::new(
                    p,
                    Vector3d::new(0.0, 0.0, 1.0),
                    t,
                    &self.material,
                    ray,
                    u,
                    v,
                ))
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_transform(&self) -> Option<&InversableTransform> {
        Some(&self.transform)
    }

    fn get_bounding_box(&self) -> AABB {
        AABB {
            min_p: Vector3d::new(self.x0, self.y0, -0.0001),
            max_p: Vector3d::new(self.x1, self.y1, 0.0001),
        }
        .transform(&self.transform.direct)
    }
}

#[derive(Debug)]
pub struct Cube {
    min_p: Vector3d,
    max_p: Vector3d,
    name: String,
    transform: InversableTransform,
    material: Arc<Box<dyn Material>>,
}

impl Cube {
    pub fn new(
        name: String,
        transform: InversableTransform,
        material: Arc<Box<dyn Material>>,
    ) -> Self {
        Self {
            min_p: Vector3d::new(-1.0, -1.0, -1.0),
            max_p: Vector3d::new(1.0, 1.0, 1.0),
            name,
            transform,
            material: material.clone(),
        }
    }
}

impl Shape for Cube {
    fn ray_intersect(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
        let t_lower = (&self.min_p - &ray.origin).divide(&ray.direction);
        let t_upper = (&self.max_p - &ray.origin).divide(&ray.direction);

        let t_mins = t_lower.min(&t_upper);
        let t_maxes = t_lower.max(&t_upper);

        let t_box_min = t_mins.max_component().max(min_t);
        let t_box_max = t_maxes.min_component().min(max_t);

        if t_box_min > t_box_max || t_box_min > max_t {
            None
        } else {
            let p = &ray.origin + t_box_min * &ray.direction;
            let (normal, u, v) = {
                // let p_abs = Vector3d::new(
                //     (p.x - (self.min_p.x + self.max_p.x) / 2.0).abs(),
                //     (p.y - (self.min_p.y + self.max_p.y) / 2.0).abs(),
                //     (p.z - (self.min_p.z + self.max_p.z) / 2.0).abs()
                // );
                let p_abs = p.abs();
                let max_c = p_abs.max_component();

                if max_c == p_abs.x {
                    (Vector3d::new(p.x, 0.0, 0.0), p.y, p.z)
                } else if max_c == p_abs.y {
                    (Vector3d::new(0.0, p.y, 0.0), p.x, p.z)
                } else if max_c == p_abs.z {
                    (Vector3d::new(0.0, 0.0, p.z), p.x, p.y)
                } else {
                    panic!("Unexpected max_c value: {}", max_c);
                }
            };
            Some(RayHit::new(p, normal, t_box_min, &self.material, ray, u, v))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_transform(&self) -> Option<&InversableTransform> {
        Some(&self.transform)
    }

    fn get_bounding_box(&self) -> AABB {
        AABB {
            min_p: self.min_p,
            max_p: self.max_p,
        }
        .transform(&self.transform.direct)
    }
}

#[derive(Debug)]
pub struct Sphere {
    name: String,
    transform: InversableTransform,
    material: Arc<Box<dyn Material>>,
    inverse_normal: bool,
}

impl Sphere {
    pub fn new(
        name: String,
        transform: InversableTransform,
        material: Arc<Box<dyn Material>>,
        inverse_normal: bool,
    ) -> Self {
        Self {
            name,
            transform,
            material,
            inverse_normal,
        }
    }
}

impl Shape for Sphere {
    fn ray_intersect(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
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
        let normal = if self.inverse_normal { -p } else { p.clone() };

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

    fn get_bounding_box(&self) -> AABB {
        AABB {
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
        .transform(&self.transform.direct)
    }
}

#[derive(Debug)]
struct Torus {
    name: String,
    radius: f64,
    tube_radius: f64,
    transform: InversableTransform,
    material: Arc<Box<dyn Material>>,
}

impl Torus {
    fn new(
        name: String,
        radius: f64,
        tube_radius: f64,
        transform: InversableTransform,
        material: Arc<Box<dyn Material>>,
    ) -> Self {
        Self {
            name,
            transform,
            material,
            radius,
            tube_radius,
        }
    }
}

impl Shape for Torus {
    fn ray_intersect(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
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

    fn get_bounding_box(&self) -> AABB {
        let a = self.radius + self.tube_radius;
        AABB {
            min_p: Vector3d::new(-a, -a, -self.tube_radius),
            max_p: Vector3d::new(a, a, self.tube_radius),
        }
        .transform(&self.transform.direct)
    }
}

#[derive(Debug)]
struct Tooth {
    name: String,
    transform: InversableTransform,
    material: Box<dyn Material>,
}

impl Shape for Tooth {
    fn ray_intersect(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
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

    fn get_bounding_box(&self) -> AABB {
        AABB::default()
    }
}

#[derive(Debug)]
pub struct ShapeCollection {
    name: String,
    shapes: Vec<Box<dyn Shape>>,
}

impl Shape for ShapeCollection {
    fn ray_intersect(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
        // self.shapes
        //     .iter()
        //     .filter_map(|shape| shape.ray_hit(ray, min_t, max_t))
        //     .min_by(|a, b| {
        //         if approx_equal(a.distance, b.distance) {
        //             Ordering::Equal
        //         } else if a.distance < b.distance {
        //             Ordering::Less
        //         } else {
        //             Ordering::Greater
        //         }
        //     })

        let mut min_distance = max_t;
        let mut min_hit: Option<RayHit> = None;
        for shape in self.shapes.iter() {
            if let Some(hit) = shape.ray_hit(ray, min_t, min_distance) {
                min_distance = hit.distance;
                min_hit = Some(hit);
            }
        }

        min_hit
    }

    fn get_bounding_box(&self) -> AABB {
        self.shapes.iter().fold(AABB::minimum(), |mut acc, shape| {
            acc.enlarge(&shape.get_bounding_box());
            acc
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ShapeCollection {
    pub fn new(name: &str, shapes: Vec<Box<dyn Shape>>) -> Self {
        Self {
            name: name.into(),
            shapes,
        }
    }
}

#[derive(Debug)]
pub struct BvhNode {
    left: Box<dyn Shape>,
    right: Option<Box<dyn Shape>>,
    bounding_box: AABB,
}

impl Shape for BvhNode {
    fn ray_hit(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
        if self.bounding_box.ray_hit(ray, min_t, max_t) {
            self.ray_hit_transformed(ray, min_t, max_t)
        } else {
            None
        }
    }

    fn ray_intersect(&self, ray: &Ray, min_t: f64, max_t: f64) -> Option<RayHit> {
        let left_hit = self.left.ray_hit(ray, min_t, max_t);
        if self.right.is_some() {
            match left_hit {
                Some(hit) => self
                    .right
                    .as_ref()
                    .unwrap()
                    .ray_hit(ray, min_t, hit.distance)
                    .or(Some(hit)),
                None => self.right.as_ref().unwrap().ray_hit(ray, min_t, max_t),
            }
        } else {
            left_hit
        }
    }

    fn get_bounding_box(&self) -> AABB {
        self.bounding_box.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BvhNode {
    pub fn new(mut shapes: Vec<Box<dyn Shape>>) -> Self {
        let mut rng = rand::thread_rng();
        let axis = rng.gen_range(0..2);

        if axis == 0 {
            shapes.sort_by(|a, b| {
                let a_bb = a.get_bounding_box();
                let b_bb = b.get_bounding_box();

                if a_bb.min_p.x < b_bb.min_p.x {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });
        } else if axis == 1 {
            shapes.sort_by(|a, b| {
                let a_bb = a.get_bounding_box();
                let b_bb = b.get_bounding_box();

                if a_bb.min_p.y < b_bb.min_p.y {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });
        } else {
            shapes.sort_by(|a, b| {
                let a_bb = a.get_bounding_box();
                let b_bb = b.get_bounding_box();

                if a_bb.min_p.z < b_bb.min_p.z {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });
        }

        let n = shapes.len();
        let (left, right) = if n == 1 {
            let s = shapes.swap_remove(0);
            (s, None)
        } else if n == 2 {
            (shapes.swap_remove(0), Some(shapes.swap_remove(0)))
        } else {
            (
                Box::new(BvhNode::new(shapes.drain(..n / 2).collect())) as Box<dyn Shape>,
                Some(Box::new(BvhNode::new(shapes)) as Box<dyn Shape>),
            )
        };

        let aabb = if right.is_some() {
            let left_bb = left.get_bounding_box();
            let right_bb = right.as_ref().unwrap().get_bounding_box();
            left_bb.max(&right_bb)
        } else {
            left.get_bounding_box().clone()
        };

        Self {
            left,
            right,
            bounding_box: aabb,
        }
    }
}

mod json_models {
    use super::{super::json_models::ShapeJson, material::Material};
    use crate::algebra::transform::InversableTransform;
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, fmt::Debug, sync::Arc};

    fn default_false() -> bool {
        false
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Sphere {
        transform: InversableTransform,
        name: String,
        material: String,

        #[serde(default = "default_false")]
        inverse_normal: bool,
    }

    #[typetag::serde]
    impl ShapeJson for Sphere {
        fn make_shape(
            &self,
            materials: &HashMap<String, Arc<Box<dyn Material>>>,
        ) -> Box<dyn super::Shape> {
            Box::new(super::Sphere::new(
                self.name.clone(),
                self.transform.clone(),
                materials[&self.material].clone(),
                self.inverse_normal,
            ))
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Torus {
        name: String,
        radius: f64,
        tube_radius: f64,
        transform: InversableTransform,
        material: String,
    }

    #[typetag::serde]
    impl ShapeJson for Torus {
        fn make_shape(
            &self,
            materials: &HashMap<String, Arc<Box<dyn Material>>>,
        ) -> Box<dyn super::Shape> {
            Box::new(super::Torus::new(
                self.name.clone(),
                self.radius,
                self.tube_radius,
                self.transform.clone(),
                materials[&self.material].clone(),
            ))
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Rectangle {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        transform: InversableTransform,
        material: String,
    }

    #[typetag::serde]
    impl ShapeJson for Rectangle {
        fn make_shape(
            &self,
            materials: &HashMap<String, Arc<Box<dyn Material>>>,
        ) -> Box<dyn super::Shape> {
            Box::new(super::Rectangle::new(
                self.x0,
                self.y0,
                self.x1,
                self.y1,
                self.transform.clone(),
                materials[&self.material].clone(),
            ))
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Cube {
        name: String,
        transform: InversableTransform,
        material: String,
    }

    #[typetag::serde]
    impl ShapeJson for Cube {
        fn make_shape(
            &self,
            materials: &HashMap<String, Arc<Box<dyn Material>>>,
        ) -> Box<dyn super::Shape> {
            Box::new(super::Cube::new(
                self.name.clone(),
                self.transform.clone(),
                materials[&self.material].clone(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Torus;
    use crate::{
        algebra::{approx_equal, transform::InversableTransform, Vector3d},
        world::{material::EmptyMaterial, shapes::AABB, Ray, Shape},
    };
    use std::{str::FromStr, sync::Arc};

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
            material: Arc::new(Box::new(mat)),
        };
        println!("Torus: {:?}", tor);

        let hit = tor.ray_intersect(&ray, 0.001, f64::INFINITY);

        println!("{:?}", hit);
    }

    #[test]
    fn test_bound_transform() {
        let b1 = AABB {
            min_p: Vector3d::new(-1.0, -1.0, -1.0),
            max_p: Vector3d::new(1.0, 1.0, 1.0),
        };
        let transform = InversableTransform::new(
            Vector3d::new(-10.0, 5.0, 2.5),
            Vector3d::new(0.0, 0.0, 0.0),
            Vector3d::new(2.0, 2.0, 2.0),
        );
        let b2 = b1.transform(&transform.direct);

        assert!(approx_equal(-12.0, b2.min_p.x));
        assert!(approx_equal(3.0, b2.min_p.y));
        assert!(approx_equal(0.5, b2.min_p.z));
        assert!(approx_equal(-8.0, b2.max_p.x));
        assert!(approx_equal(7.0, b2.max_p.y));
        assert!(approx_equal(4.5, b2.max_p.z));
    }
}
