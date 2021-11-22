use std::{any::Any, f64::consts::PI, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::algebra::{
    approx_equal, equation::solve_quantic_equation, transform::InversableTransform, Vector3d,
};

use super::{material::Material, Ray, RayHit};

pub mod brute_forced;

#[typetag::serde(tag = "type")]
pub trait Shape: Debug + Send + Sync {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>>;

    fn as_any(&self) -> &dyn Any;
}

pub trait TransformableShape: Shape {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>>;

    fn get_transform(&self) -> &InversableTransform;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Plane {
    pub normal: Vector3d,
    pub p0: Vector3d,
    material: Box<dyn Material>,
}

#[typetag::serde]
impl Shape for Plane {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let ln = &ray.direction * &self.normal;
        if approx_equal(ln, 0.0) {
            None
        } else {
            let x = ((&self.p0 - &ray.origin) * &self.normal) / ln;
            if x < min_t || x > max_t {
                None
            } else {
                Some(RayHit::new(
                    &ray.origin + x * &ray.direction,
                    self.normal.clone(),
                    x,
                    // &self.normal * &ray.direction < 0.0,
                    &self.material,
                    ray,
                    // TODO: сделать текстурные координаты
                    0.0,
                    0.0,
                ))
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sphere {
    pub center: Vector3d,
    pub radius: f64,
    pub material: Box<dyn Material>,
}

#[typetag::serde]
impl Shape for Sphere {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let oc = &ray.origin - &self.center;
        let a = 1.0;
        let half_b = &ray.direction * &oc;
        let c = &oc * &oc - self.radius * self.radius;

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

        let p = &ray.origin + &ray.direction * x;
        let normal = (&p - &self.center) / self.radius;

        // let is_front_face = &normal * &ray.direction < 0.0;
        // if !is_front_face {
        //     normal = -normal;
        // }

        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;
        Some(RayHit::new(
            p,
            normal,
            x,
            // is_front_face,
            &self.material,
            ray,
            phi / (2.0 * PI),
            theta / PI,
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Elipsoid {
    pub center: Vector3d,
    pub radius: Vector3d,
    pub material: Box<dyn Material>,
}

#[typetag::serde]
impl Shape for Elipsoid {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let oc = &ray.origin - &self.center;
        let a = ray.direction.x * ray.direction.x / self.radius.x
            + ray.direction.y * ray.direction.y / self.radius.y
            + ray.direction.z * ray.direction.z / self.radius.z;
        let half_b = oc.x * ray.direction.x / self.radius.x
            + oc.y * ray.direction.y / self.radius.y
            + oc.z * ray.direction.z / self.radius.z;
        let c =
            oc.x * oc.x / self.radius.x + oc.y * oc.y / self.radius.y + oc.z * oc.z / self.radius.z
                - 1.0;

        //  TODO: вынести решение квадратного уравнения в отдельную функцию
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

        let p = &ray.origin + &ray.direction * x;
        let normal = (&p - &self.center) / &self.radius;

        // let is_front_face = &normal * &ray.direction < 0.0;
        // if !is_front_face {
        //     normal = -normal;
        // }
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;
        Some(RayHit::new(
            p,
            normal,
            x,
            // is_front_face,
            &self.material,
            ray,
            phi / (2.0 * PI),
            theta / PI,
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransformedSphere {
    transform: InversableTransform,
    material: Box<dyn Material>,
}

#[typetag::serde]
impl Shape for TransformedSphere {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let origin = self.transform.inverse.transform_point(&ray.origin);
        let dir = self.transform.inverse.transform_vector(&ray.direction);

        let a = &dir * &dir;
        let half_b = &dir * &origin;
        let c = &origin * &origin - 1.0;

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

        let p = &origin + &dir * x;
        let normal = p.clone();

        // let theta = (p.z).acos();
        // let phi = (p.y).atan2(p.x) + PI;
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;
        Some(RayHit::new(
            self.transform.direct.transform_point(&p),
            self.transform.inverse.transform_normal(&normal),
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
}

#[derive(Serialize, Deserialize, Debug)]
struct Torus {
    radius: f64,
    tube_radius: f64,
    transform: InversableTransform,
    material: Box<dyn Material>,
}

#[typetag::serde]
impl Shape for Torus {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let origin = self.transform.inverse.transform_point(&ray.origin);
        let dir = self.transform.inverse.transform_vector(&ray.direction);

        let t = 4.0 * self.radius * self.radius;
        let g = t * (dir.x * dir.x + dir.y * dir.y);
        let h = 2.0 * t * (origin.x * dir.x + origin.y * dir.y);
        let i = t * (origin.x * origin.x + origin.y * origin.y);
        let j = dir.squared_length();
        let k = 2.0 * (&origin * &dir);
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
            let p = &origin + min_root * &dir;
            let normal = &p - Vector3d::new(p.x, p.y, 0.0).normalize() * self.radius;
            let theta = (p.z / self.tube_radius).asin();
            let phi = (p.z / (self.radius + self.tube_radius * theta.cos())).acos() + PI;
            let p = self.transform.direct.transform_point(&p);

            Some(RayHit::new(
                p,
                self.transform.inverse.transform_normal(&normal),
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
}

#[derive(Serialize, Deserialize, Debug)]
struct Tooth {
    transform: InversableTransform,
    material: Box<dyn Material>,
}

#[typetag::serde]
impl Shape for Tooth {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let o = self.transform.inverse.transform_point(&ray.origin);
        let dir = self.transform.inverse.transform_vector(&ray.direction);

        let dir2 = dir.powi(2);
        let o2 = o.powi(2);

        let a = &dir2 * &dir2;
        let b = 4.0 * (dir.powi(3) * &o);
        let c = 6.0 * (dir2 * &o2) - o.squared_length();
        let d = 4.0 * (&dir * o.powi(3)) - 2.0 * (&o * &dir);
        let e = &o2 * &o2 - &o * &o;

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
            let p = &o + min_root * &dir;

            let normal = Vector3d::new(
                4.0 * p.x.powi(3) - 2.0 * p.x,
                4.0 * p.y.powi(3) - 2.0 * p.y,
                4.0 * p.z.powi(3) - 2.0 * p.z,
            );

            Some(RayHit::new(
                self.transform.direct.transform_point(&p),
                self.transform.inverse.transform_normal(&normal),
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
}

#[cfg(test)]
mod tests {
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
