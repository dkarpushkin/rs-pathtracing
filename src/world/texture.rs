use std::{f64::consts::PI, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::algebra::{noise::Perlin, Vector3d};

#[typetag::serde(tag = "type")]
pub trait Texture: Debug + Send + Sync {
    fn value(&self, u: f64, v: f64, p: &Vector3d) -> Vector3d;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolidColor {
    pub color: Vector3d,
}

#[typetag::serde]
impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Vector3d) -> Vector3d {
        self.color.clone()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckerTexture {
    pub odd: Box<dyn Texture>,
    pub even: Box<dyn Texture>,
    pub multipliers: Vector3d,
}

impl CheckerTexture {
    pub fn new(odd: Vector3d, even: Vector3d, multipliers: Vector3d) -> Self {
        Self {
            odd: Box::new(SolidColor { color: odd }),
            even: Box::new(SolidColor { color: even }),
            multipliers,
        }
    }
}

#[typetag::serde]
impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Vector3d) -> Vector3d {
        let sines = (self.multipliers.x * p.x).sin()
            * (self.multipliers.y * p.y).sin()
            * (self.multipliers.z * p.z).sin();
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NoiseTexture {
    #[serde(skip_serializing, skip_deserializing)]
    noise: Perlin,
    scale: f64,
}

#[typetag::serde]
impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Vector3d) -> Vector3d {
        // Vector3d::new(1.0, 1.0, 1.0) * 0.5 * (1.0 + self.noise.noise(&(p * self.scale)))
        // Vector3d::new(1.0, 1.0, 1.0) * self.noise.turb(&(p * self.scale), 7)
        0.5 * (1.0 + (self.scale * p.z + 10.0 * self.noise.turb(&p, 7)).sin())
            * Vector3d::new(1.0, 1.0, 1.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UVChecker {
    pub odd: Box<dyn Texture>,
    pub even: Box<dyn Texture>,
    pub multipliers: (f64, f64),
}

#[typetag::serde]
impl Texture for UVChecker {
    fn value(&self, u: f64, v: f64, p: &Vector3d) -> Vector3d {
        let sines = (v * self.multipliers.0 * PI).sin() * (u * self.multipliers.1 * PI).sin();
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}
