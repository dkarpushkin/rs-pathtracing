use std::ops::Range;

use itertools::Itertools;
use rand::{Rng, prelude::ThreadRng};

use crate::{algebra::Vector3d, world::Ray};

use super::Camera;


#[derive(Debug)]
pub struct MultisamplerRayCaster {
    // camera: Arc<RwLock<Camera>>,
    camera_position: Vector3d,
    camera_right: Vector3d,
    camera_up: Vector3d,

    left_top: Vector3d,
    coords_iter: itertools::Product<Range<u32>, Range<u32>>,
    pixel_resolution: f64,
    rng: ThreadRng,
    samples_number: u32,
}

impl MultisamplerRayCaster {
    pub fn new(camera: &Camera, samples_number: u32) -> Self {
        let center = &camera.position + camera.focal_length * &camera.direction;
        Self {
            left_top: &camera.position + camera.focal_length * &camera.direction
                - &camera.rigth * (camera.viewport_width / 2.0)
                + &camera.up * (camera.viewport_height / 2.0),
            coords_iter: (0..camera.image.height).cartesian_product(0..camera.image.width),
            pixel_resolution: camera.viewport_width / camera.image.width as f64,
            rng: rand::thread_rng(),
            samples_number: samples_number,
            camera_position: camera.position.clone(),
            camera_right: camera.rigth.clone(),
            camera_up: camera.up.clone(),
        }
    }

    pub fn get_ray(&self, x: f64, y: f64) -> Ray {
        let dir = &self.left_top + (self.pixel_resolution * x) * &self.camera_right
            - (self.pixel_resolution * y) * &self.camera_up;
        Ray::new(self.camera_position.clone(), dir - &self.camera_position)
    }

    pub fn get_pixel_sample(&mut self, x: u32, y: u32) -> Vec<Ray> {
        (0..self.samples_number)
            .map(|_| {
                let u: f64 = self.rng.gen();
                let v: f64 = self.rng.gen();

                self.get_ray(x as f64 + u, y as f64 + v)
            })
            .collect()
    }
}

impl Iterator for MultisamplerRayCaster {
    type Item = (u32, u32, Vec<Ray>);

    fn next(&mut self) -> Option<Self::Item> {
        let (y, x) = self.coords_iter.next()?;
        let samples = (0..self.samples_number)
            .map(|_| {
                let u: f64 = self.rng.gen();
                let v: f64 = self.rng.gen();

                let dir = &self.left_top
                    + (self.pixel_resolution * (x as f64 + u)) * &self.camera_right
                    - (self.pixel_resolution * (y as f64 + v)) * &self.camera_up;
                Ray::new(self.camera_position.clone(), dir - &self.camera_position)
            })
            .collect();

        Some((x, y, samples))
    }
}

pub struct SinglesamplerRayCaster<'a> {
    camera: &'a Camera,
    left_bottom: Vector3d,
    coords_iter: itertools::Product<Range<u32>, Range<u32>>,
    pixel_resolution: f64,
}

impl<'a> SinglesamplerRayCaster<'a> {
    pub fn new(camera: &'a Camera) -> Self {
        let center = &camera.position + camera.focal_length * &camera.direction;
        Self {
            camera: camera,
            left_bottom: center
                - &camera.rigth * (camera.viewport_width / 2.0)
                - &camera.up * (camera.viewport_height / 2.0),
            coords_iter: (0..camera.image.height).cartesian_product(0..camera.image.width),
            pixel_resolution: camera.viewport_width / camera.image.width as f64,
        }
    }
}

impl<'a> Iterator for SinglesamplerRayCaster<'a> {
    type Item = (u32, u32, Ray);

    fn next(&mut self) -> Option<Self::Item> {
        let (v, u) = self.coords_iter.next()?;

        let dir = &self.left_bottom
            + (self.pixel_resolution * (u as f64 + 0.5)) * &self.camera.rigth
            + (self.pixel_resolution * (v as f64 + 0.5)) * &self.camera.up;

        let ray = Ray::new(self.camera.position.clone(), dir);

        Some((u, v, ray))
    }
}
