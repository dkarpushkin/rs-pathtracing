use std::ops::Range;

use itertools::Itertools;
use rand::{prelude::ThreadRng, Rng};

use crate::{algebra::Vector3d, world::ray::Ray};

use super::Camera;

#[derive(Debug, Clone)]
pub struct ImageParams {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct MultisamplerRayCaster {
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
    pub fn new(camera: &Camera, img_params: &ImageParams, samples_number: u32) -> Self {
        let center = &camera.position + camera.focal_length * &camera.direction;
        let aspect_ratio = img_params.width as f64 / img_params.height as f64;
        let viewport_width = (camera.fov / 2.0).tan() * camera.focal_length * 2.0;
        let viewport_height = viewport_width / aspect_ratio;
        let coords_iter = (0..img_params.height).cartesian_product(0..img_params.width);
        
        Self {
            left_top: center - &camera.rigth * (viewport_width / 2.0)
                + &camera.up * (viewport_height / 2.0),
            coords_iter: coords_iter,
            pixel_resolution: viewport_width / img_params.width as f64,
            rng: rand::thread_rng(),
            samples_number: samples_number,
            camera_position: camera.position.clone(),
            camera_right: camera.rigth.clone(),
            camera_up: camera.up.clone(),
        }
    }

    pub fn partial(
        camera: &Camera,
        whole_image: ImageParams,
        from: (u32, u32),
        partial_image: ImageParams,
        samples_number: u32,
    ) -> Self {
        let center = &camera.position + camera.focal_length * &camera.direction;
        let aspect_ratio = whole_image.width as f64 / whole_image.height as f64;
        let viewport_width = (camera.fov / 2.0).tan() * camera.focal_length * 2.0;
        let viewport_height = viewport_width / aspect_ratio;
        let coords_iter =
            (from.0..partial_image.height).cartesian_product(from.1..partial_image.width);

        Self {
            left_top: center - &camera.rigth * (viewport_width / 2.0)
                + &camera.up * (viewport_height / 2.0),
            coords_iter,
            pixel_resolution: viewport_width / whole_image.width as f64,
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

    /// Get a reference to the multisampler ray caster's pixel resolution.
    pub fn pixel_resolution(&self) -> f64 {
        self.pixel_resolution
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.coords_iter.size_hint()
    }
}

impl ExactSizeIterator for MultisamplerRayCaster {}

pub struct SinglesamplerRayCaster<'a> {
    camera: &'a Camera,
    left_bottom: Vector3d,
    coords_iter: itertools::Product<Range<u32>, Range<u32>>,
    pixel_resolution: f64,
}

impl<'a> SinglesamplerRayCaster<'a> {
    pub fn new(camera: &'a Camera, img_params: ImageParams) -> Self {
        let center = &camera.position + camera.focal_length * &camera.direction;
        let aspect_ratio = img_params.width as f64 / img_params.height as f64;
        let viewport_width = (camera.fov / 2.0).tan() * camera.focal_length * 2.0;
        let viewport_height = viewport_width / aspect_ratio;
        Self {
            camera: camera,
            left_bottom: center
                - &camera.rigth * (viewport_width / 2.0)
                - &camera.up * (viewport_height / 2.0),
            coords_iter: (0..img_params.height).cartesian_product(0..img_params.width),
            pixel_resolution: viewport_width / img_params.width as f64,
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
