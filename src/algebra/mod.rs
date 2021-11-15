use std::{
    fmt::Display,
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub},
};

use rand::Rng;
use serde::{Deserialize, Serialize};

pub mod matrix;

pub fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-15
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Vector3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3d {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3d { x, y, z }
    }

    pub fn random(min: f64, max: f64) -> Vector3d {
        let mut rng = rand::thread_rng();
        Vector3d {
            x: rng.gen_range(min..=max),
            y: rng.gen_range(min..=max),
            z: rng.gen_range(min..=max),
        }
    }

    pub fn random_in_sphere(radius: f64) -> Vector3d {
        loop {
            let random_vector = Vector3d::random(-radius, radius);
            if random_vector.squared_length() <= radius * radius {
                break random_vector;
            }
        }
    }

    pub fn random_in_unit_sphere() -> Vector3d {
        loop {
            let random_vector = Vector3d::random(-1.0, 1.0);
            if random_vector.squared_length() <= 1.0 {
                break random_vector;
            }
        }
    }

    pub fn random_unit() -> Vector3d {
        // let mut rng = rand::thread_rng();
        // loop {
        //     let random_vector = Vector3d {
        //         x: rng.gen_range(-1.0..1.0),
        //         y: rng.gen_range(-1.0..1.0),
        //         z: rng.gen_range(-1.0..1.0),
        //     };
        //     let len = random_vector.squared_length();
        //     if len <= 1.0 {
        //         break random_vector / len;
        //     }
        // }

        Vector3d::random_in_unit_sphere().normalize()
    }

    pub fn random_in_hemisphere(normal: &Vector3d) -> Vector3d {
        let random_vector = Vector3d::random_unit();
        if &random_vector * normal > 0.0 {
            random_vector
        } else {
            -random_vector
        }
    }

    pub fn cross(&self, other: &Vector3d) -> Vector3d {
        Vector3d {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline]
    pub fn normalize(&self) -> Vector3d {
        self / self.length()
    }

    #[inline]
    pub fn squared_length(&self) -> f64 {
        self * self
    }

    #[inline]
    pub fn length(&self) -> f64 {
        self.squared_length().sqrt()
    }

    pub fn reflect(&self, normal: &Vector3d) -> Vector3d {
        let b = (self * normal) * normal;
        self - (2.0 * b)
    }

    pub fn refract(&self, normal: &Vector3d, ratio: f64) -> Vector3d {
        let cos_theta = -self * normal;
        let r_out_perp = ratio * (self + cos_theta * normal);
        let r_out_parallel = -((1.0 - r_out_perp.squared_length()).abs().sqrt()) * normal;

        r_out_perp + r_out_parallel
    }

    pub fn product(&self, other: &Vector3d) -> Vector3d {
        Vector3d {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }

    pub fn is_zero(&self) -> bool {
        approx_equal(self.x, 0.0) && approx_equal(self.y, 0.0) && approx_equal(self.z, 0.0)
    }
}

impl Display for Vector3d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl PartialEq for Vector3d {
    fn eq(&self, other: &Self) -> bool {
        approx_equal(self.x, other.x)
            && approx_equal(self.y, other.y)
            && approx_equal(self.z, other.z)
    }
}

impl PartialEq<Vector3d> for &Vector3d {
    fn eq(&self, other: &Vector3d) -> bool {
        approx_equal(self.x, other.x)
            && approx_equal(self.y, other.y)
            && approx_equal(self.z, other.z)
    }
}

impl Add<Vector3d> for Vector3d {
    type Output = Vector3d;

    fn add(self, rhs: Vector3d) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<&Vector3d> for Vector3d {
    type Output = Vector3d;

    fn add(self, rhs: &Vector3d) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<Vector3d> for &Vector3d {
    type Output = Vector3d;

    fn add(self, rhs: Vector3d) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<&Vector3d> for &Vector3d {
    type Output = Vector3d;

    fn add(self, rhs: &Vector3d) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<Vector3d> for Vector3d {
    type Output = Vector3d;

    fn sub(self, rhs: Vector3d) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<&Vector3d> for Vector3d {
    type Output = Vector3d;

    fn sub(self, rhs: &Vector3d) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<&Vector3d> for &Vector3d {
    type Output = Vector3d;

    fn sub(self, rhs: &Vector3d) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<Vector3d> for &Vector3d {
    type Output = Vector3d;

    fn sub(self, rhs: Vector3d) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<Vector3d> for Vector3d {
    type Output = f64;

    fn mul(self, rhs: Vector3d) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Mul<&Vector3d> for Vector3d {
    type Output = f64;

    fn mul(self, rhs: &Vector3d) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Mul<&Vector3d> for &Vector3d {
    type Output = f64;

    fn mul(self, rhs: &Vector3d) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Mul<f64> for Vector3d {
    type Output = Vector3d;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output {
            x: rhs * self.x,
            y: rhs * self.y,
            z: rhs * self.z,
        }
    }
}

impl Mul<f64> for &Vector3d {
    type Output = Vector3d;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output {
            x: rhs * self.x,
            y: rhs * self.y,
            z: rhs * self.z,
        }
    }
}

impl Mul<Vector3d> for f64 {
    type Output = Vector3d;

    fn mul(self, rhs: Vector3d) -> Self::Output {
        Self::Output {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
        }
    }
}

impl Mul<&Vector3d> for f64 {
    type Output = Vector3d;

    fn mul(self, rhs: &Vector3d) -> Self::Output {
        Self::Output {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
        }
    }
}

impl Div<f64> for Vector3d {
    type Output = Vector3d;

    fn div(self, rhs: f64) -> Self::Output {
        Vector3d {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Div<f64> for &Vector3d {
    type Output = Vector3d;

    fn div(self, rhs: f64) -> Self::Output {
        Vector3d {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl AddAssign for Vector3d {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl AddAssign<&Vector3d> for Vector3d {
    fn add_assign(&mut self, rhs: &Vector3d) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Neg for Vector3d {
    type Output = Vector3d;

    fn neg(self) -> Self::Output {
        Vector3d {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Neg for &Vector3d {
    type Output = Vector3d;

    fn neg(self) -> Self::Output {
        Vector3d {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<'a> Sum<Vector3d> for Vector3d {
    fn sum<I: Iterator<Item = Vector3d>>(iter: I) -> Self {
        let mut acc = Vector3d::new(0.0, 0.0, 0.0);
        for e in iter {
            acc += e;
        }
        acc
    }
}

impl<'a> Sum<&'a Vector3d> for Vector3d {
    fn sum<I: Iterator<Item = &'a Vector3d>>(iter: I) -> Self {
        let mut acc = Vector3d::new(0.0, 0.0, 0.0);
        for e in iter {
            acc += e;
        }
        acc
    }
}