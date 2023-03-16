use rand::Rng;

use super::{
    Ray, RayHit,
};
use crate::algebra::{
    transform::{InversableTransform, Transform},
    Vector3d,
};
use std::{any::Any, fmt::Debug, ops::Index, sync::Arc};

pub mod ray_marching;
pub mod shapes;

pub use shapes::*;

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

pub type ShapePtr = Arc<Box<dyn Shape>>;

#[allow(dead_code)]
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
    left: ShapePtr,
    right: Option<ShapePtr>,
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
    pub fn new(shapes: &[ShapePtr]) -> Self {
        let mut rng = rand::thread_rng();
        let axis = rng.gen_range(0..2);

        // if axis == 0 {
        //     shapes.sort_by(|a, b| {
        //         let a_bb = a.get_bounding_box();
        //         let b_bb = b.get_bounding_box();

        //         if a_bb.min_p.x < b_bb.min_p.x {
        //             std::cmp::Ordering::Less
        //         } else {
        //             std::cmp::Ordering::Greater
        //         }
        //     });
        // } else if axis == 1 {
        //     shapes.sort_by(|a, b| {
        //         let a_bb = a.get_bounding_box();
        //         let b_bb = b.get_bounding_box();

        //         if a_bb.min_p.y < b_bb.min_p.y {
        //             std::cmp::Ordering::Less
        //         } else {
        //             std::cmp::Ordering::Greater
        //         }
        //     });
        // } else {
        //     shapes.sort_by(|a, b| {
        //         let a_bb = a.get_bounding_box();
        //         let b_bb = b.get_bounding_box();

        //         if a_bb.min_p.z < b_bb.min_p.z {
        //             std::cmp::Ordering::Less
        //         } else {
        //             std::cmp::Ordering::Greater
        //         }
        //     });
        // }

        let n = shapes.len();
        let (left, right) = if n == 1 {
            // Only one non-BVH shape
            let s = Arc::clone(&shapes[0]);
            (s, None)
        } else if n == 2 {
            // Both are non-BVH shapes
            (Arc::clone(&shapes[0]), Some(Arc::clone(&shapes[1])))
        } else {
            // both are BvhNode
            (
                Arc::new(Box::new(BvhNode::new(&shapes[(..n / 2)])) as Box<dyn Shape>),
                Some(Arc::new(Box::new(BvhNode::new(&shapes[(n / 2..)])) as Box<dyn Shape>)),
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
            left: Arc::clone(&left),
            right,
            bounding_box: aabb,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        algebra::{approx_equal, transform::InversableTransform, Vector3d},
        world::shapes::AABB,
    };

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
