use super::{super::Material, Shape};
use crate::{
    algebra::{
        approx_equal, equation::solve_quadratic_equation, transform::InversableTransform, Vector3d,
    },
    world::{Ray, RayHit},
};
use serde::{Deserialize, Serialize};
use std::{any::Any, fmt::Debug};

#[derive(Serialize, Deserialize, Debug)]
struct BruteForcableShape {
    transform: InversableTransform,
    material: Box<dyn Material>,
    shape: Box<dyn BruteForceShape>,
    step: f64,
}

#[typetag::serde]
impl Shape for BruteForcableShape {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let origin = self.transform.inverse.transform_point(&ray.origin);
        let dir = self.transform.inverse.transform_vector(&ray.direction);

        let (start, end) = self.shape.intersect_bound(&origin, &dir)?;
        let mut step = self.step;

        let mut t = start;
        let mut p = &origin + t * &dir;
        let mut r = self.shape.shape_func(&p);
        'outer: for _ in 0..4 {
            loop {
                if t > end || t < start {
                    return None;
                }
                t += step;
                p += step * &dir;

                let next = self.shape.shape_func(&p);
                if approx_equal(next, 0.0) {
                    break 'outer;
                }

                if (r < 0.0 && next > 0.0) || (r > 0.0 && next < 0.0) {
                    step *= -0.01;
                    r = next;
                    break;
                }

                r = next;
            }
        }

        if t < min_t || t > max_t {
            return None;
        }

        let p = &origin + &dir * t;
        let normal = self.shape.gradient(&p);
        let (u, v) = self.shape.uv(&p);

        Some(RayHit::new(
            self.transform.direct.transform_point(&p),
            self.transform.inverse.transform_normal(&normal),
            t,
            &self.material,
            ray,
            u,
            v,
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[typetag::serde(tag = "type")]
trait BruteForceShape: Debug + Send + Sync {
    fn shape_func(&self, p: &Vector3d) -> f64;
    fn intersect_bound(&self, origin: &Vector3d, dir: &Vector3d) -> Option<(f64, f64)>;
    fn gradient(&self, p: &Vector3d) -> Vector3d;
    fn uv(&self, p: &Vector3d) -> (f64, f64);
}

#[derive(Serialize, Deserialize, Debug)]
struct Heart {
    sphere_radius: f64,
}

#[typetag::serde]
impl BruteForceShape for Heart {
    fn intersect_bound(&self, origin: &Vector3d, dir: &Vector3d) -> Option<(f64, f64)> {
        let (x1, x2) = solve_quadratic_equation(
            dir * dir,
            dir * origin,
            origin * origin - self.sphere_radius * self.sphere_radius,
        )?;

        if x1 < 0.0 && x2 < 0.0 {
            None
        } else {
            Some((x1.max(0.0), x2.max(0.0)))
        }
    }

    fn shape_func(&self, p: &Vector3d) -> f64 {
        let x2 = p.x * p.x;
        let y2 = p.y * p.y;
        let z2 = p.z * p.z;
        let z3 = z2 * p.z;

        let a = x2 + (9.0 / 4.0) * y2 + z2 - 1.0;
        a * a * a - x2 * z3 - (9.0 / 80.0) * y2 * z3
    }

    fn gradient(&self, p: &Vector3d) -> Vector3d {
        let a = p.x * p.x + (9.0 / 4.0) * p.y * p.y + p.z * p.z - 1.0;
        let a = 3.0 * a * a;
        let z2 = p.z * p.z;
        let z3 = z2 * p.z;

        Vector3d::new(
            2.0 * p.x * (a - z3),
            (9.0 / 2.0) * p.y * (a - 0.05 * z3),
            2.0 * p.z * (a - p.z * (1.5 * p.x * p.x + (27.0 / 40.0) * p.y * p.y)),
        )
    }

    fn uv(&self, _p: &Vector3d) -> (f64, f64) {
        (0.0, 0.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Sine {
    a: f64,
    sphere_radius: f64,
}

#[typetag::serde]
impl BruteForceShape for Sine {
    fn shape_func(&self, p: &Vector3d) -> f64 {
        self.a
            * self.a
            * (p.x - p.y - p.z)
            * (p.x + p.y - p.z)
            * (p.x - p.y + p.z)
            * (p.x + p.y + p.z)
            + 4.0 * p.x * p.x * p.y * p.y * p.z * p.z
    }

    fn intersect_bound(&self, origin: &Vector3d, dir: &Vector3d) -> Option<(f64, f64)> {
        solve_quadratic_equation(
            dir * dir,
            dir * origin,
            origin * origin - self.sphere_radius * self.sphere_radius,
        )
    }

    fn gradient(&self, p: &Vector3d) -> Vector3d {
        let x2 = p.x * p.x;
        let y2 = p.y * p.y;
        let z2 = p.z * p.z;
        let a2 = self.a * self.a;
        Vector3d::new(
            4.0 * p.x * (a2 * (x2 - y2 - z2) + 2.0 * y2 * z2),
            8.0 * x2 * p.y * z2 - 4.0 * a2 * p.y * (x2 - y2 + z2),
            8.0 * x2 * y2 * p.z - 4.0 * a2 * p.z * (x2 + y2 - z2),
        )
    }

    fn uv(&self, p: &Vector3d) -> (f64, f64) {
        (0.0, 0.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Star {
    a: f64,
    sphere_radius: f64,
}

#[typetag::serde]
impl BruteForceShape for Star {
    fn shape_func(&self, p: &Vector3d) -> f64 {
        let x2 = p.x * p.x;
        let y2 = p.y * p.y;
        let z2 = p.z * p.z;
        let c = x2 + y2 + z2 - 1.0;
        self.a * (x2 * y2 + x2 * z2 + y2 * z2) + (c * c * c)
    }

    fn intersect_bound(&self, origin: &Vector3d, dir: &Vector3d) -> Option<(f64, f64)> {
        solve_quadratic_equation(
            dir * dir,
            dir * origin,
            origin * origin - self.sphere_radius * self.sphere_radius,
        )
    }

    fn gradient(&self, p: &Vector3d) -> Vector3d {
        let x2 = p.x * p.x;
        let y2 = p.y * p.y;
        let z2 = p.z * p.z;
        let c = x2 + y2 + z2 - 1.0;
        Vector3d::new(
            2.0 * self.a * p.x * (y2 + z2) + 6.0 * p.x * c * c,
            2.0 * self.a * p.y * (x2 + z2) + 6.0 * p.y * c * c,
            2.0 * self.a * p.z * (x2 + y2) + 6.0 * p.z * c * c,
        )
    }

    fn uv(&self, p: &Vector3d) -> (f64, f64) {
        (0.0, 0.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DupinCyclide {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    sphere_radius: f64,
}

#[typetag::serde]
impl BruteForceShape for DupinCyclide {
    fn shape_func(&self, p: &Vector3d) -> f64 {
        let b2 = self.b * self.b;
        let e = p.x * p.x + p.y * p.y + p.z * p.z + b2 - self.d * self.d;
        let f = self.a * p.x - self.c * self.d;
        e * e - 4.0 * (f * f + b2 * p.y * p.y)
    }

    fn intersect_bound(&self, origin: &Vector3d, dir: &Vector3d) -> Option<(f64, f64)> {
        solve_quadratic_equation(
            dir * dir,
            dir * origin,
            origin * origin - self.sphere_radius * self.sphere_radius,
        )
    }

    fn gradient(&self, p: &Vector3d) -> Vector3d {
        let b2 = self.b * self.b;
        let e = 4.0 * (p.x * p.x + p.y * p.y + p.z * p.z + b2 - self. d * self.d);
        Vector3d {
            x: e * p.x - 8.0 * self.a * (self.a * p.x - self.c * self.d),
            y: e * p.y - 8.0 * b2 * p.y,
            z: e * p.z
        }
    }

    fn uv(&self, p: &Vector3d) -> (f64, f64) {
        (0.0, 0.0)
    }
}