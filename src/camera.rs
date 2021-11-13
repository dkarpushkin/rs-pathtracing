use std::{fmt::Display, ops::Range};

use itertools::Itertools;
use rand::{prelude::ThreadRng, Rng};

use crate::{algebra::Vector3d, world::Ray};

#[derive(Debug, Clone)]
pub struct ImageParams {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct Camera {
    //  user defined
    position: Vector3d,
    direction: Vector3d,
    up: Vector3d,
    fov: f64,
    focal_length: f64,
    image: ImageParams,

    //  autogenerated
    rigth: Vector3d,
    aspect_ratio: f64,
    viewport_width: f64,
    viewport_height: f64,
}

impl Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pos: {}; dir: {}; up: {}; right: {}; fov: {}; focal_ln: {}",
            self.position, self.direction, self.up, self.rigth, self.fov, self.focal_length
        )
    }
}

impl Camera {
    pub fn new(
        position: &Vector3d,
        direction: &Vector3d,
        up_vector: &Vector3d,
        img_params: &ImageParams,
        focal_length: f64,
        fov: f64,
    ) -> Self {
        let aspect_ratio = img_params.width as f64 / img_params.height as f64;
        let right_vec = direction.cross(up_vector).normalize();
        let viewport_width = (fov / 2.0).tan() * focal_length * 2.0;
        Self {
            position: position.clone(),
            direction: direction.normalize(),
            up: right_vec.cross(direction).normalize(),
            rigth: right_vec,
            fov: fov,
            focal_length: focal_length,
            image: img_params.clone(),
            aspect_ratio: aspect_ratio,

            viewport_width: viewport_width,
            viewport_height: viewport_width / aspect_ratio,
        }
    }

    /// Get a reference to the camera's image.
    pub fn image(&self) -> &ImageParams {
        &self.image
    }

    /// Set the camera's image params and change the aspect ratio.
    pub fn set_image(&mut self, image: ImageParams) {
        self.aspect_ratio = image.width as f64 / image.height as f64;
        self.viewport_height = self.viewport_width / self.aspect_ratio;
        self.image = image;
    }

    /// Get a reference to the camera's fov.
    pub fn fov(&self) -> f64 {
        self.fov
    }

    /// Set the camera's fov.
    pub fn set_fov(&mut self, fov: f64) {
        self.fov = fov;
        self.viewport_width = (fov / 2.0).tan() * self.focal_length * 2.0;
        self.viewport_height = self.viewport_width / self.aspect_ratio;
    }

    /// Get a reference to the camera's focal length.
    pub fn focal_length(&self) -> f64 {
        self.focal_length
    }

    /// Set the camera's focal length.
    pub fn set_focal_length(&mut self, focal_length: f64) {
        self.focal_length = focal_length;
        self.viewport_width = (self.fov / 2.0).tan() * self.focal_length * 2.0;
        self.viewport_height = self.viewport_width / self.aspect_ratio;
    }

    /// Get a reference to the camera's position.
    pub fn position(&self) -> &Vector3d {
        &self.position
    }

    /// Set the camera's position.
    pub fn set_position(&mut self, position: Vector3d) {
        self.position = position;
    }

    /// Get a reference to the camera's direction.
    pub fn direction(&self) -> &Vector3d {
        &self.direction
    }

    /// Set the camera's direction.
    pub fn set_direction(&mut self, direction: Vector3d) {
        self.direction = direction.normalize();
        self.rigth = self.direction.cross(&self.up).normalize();
        self.up = self.rigth.cross(&self.direction).normalize();
    }

    /// Get a reference to the camera's up.
    pub fn up(&self) -> &Vector3d {
        &self.up
    }

    /// Set the camera's up.
    pub fn set_up(&mut self, up: Vector3d) {
        self.up = up;
        self.rigth = self.direction.cross(&self.up).normalize();
        self.up = self.rigth.cross(&self.direction).normalize();
    }

    /// Get a reference to the camera's aspect ratio.
    pub fn aspect_ratio(&self) -> f64 {
        self.aspect_ratio
    }

    /// Get a reference to the camera's rigth.
    pub fn rigth(&self) -> &Vector3d {
        &self.rigth
    }

    /// Get a reference to the camera's viewport width.
    pub fn viewport_width(&self) -> f64 {
        self.viewport_width
    }

    /// Get a reference to the camera's viewport height.
    pub fn viewport_height(&self) -> f64 {
        self.viewport_height
    }

    pub fn transfer(&mut self, vertical: f64, horizontal: f64, forward: f64) {
        if vertical != 0.0 {
            self.position += &self.up * vertical;
        }
        if horizontal != 0.0 {
            self.position += &self.rigth * horizontal;
        }
        if forward != 0.0 {
            self.position += &self.direction * forward;
        }
    }

    pub fn rotate_local(&mut self, vertical: f64, horizontal: f64) {
        if vertical != 0.0 {
            self.direction += &self.up * vertical;
        }
        if horizontal != 0.0 {
            self.direction += &self.rigth * horizontal;
        }

        self.direction = self.direction.normalize();
        self.rigth = self.direction.cross(&self.up).normalize();
        self.up = self.rigth.cross(&self.direction).normalize();
    }

    pub fn rotate_global(&mut self, xz: f64, yz: f64, xy: f64) {
        if xz != 0.0 {
            self.direction.x += xz;
        }
        if yz != 0.0 {
            self.direction.y += yz;
        }
        if xy != 0.0 {
            self.up.x += xy;
        }

        self.direction = self.direction.normalize();
        self.rigth = self.direction.cross(&self.up).normalize();
        self.up = self.rigth.cross(&self.direction).normalize();
    }
}

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
            left_top: center
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
}

impl Iterator for MultisamplerRayCaster {
    type Item = (u32, u32, Vec<Ray>);

    fn next(&mut self) -> Option<Self::Item> {
        let (v, u) = self.coords_iter.next()?;
        let samples = (0..self.samples_number)
            .map(|_| {
                let x: f64 = self.rng.gen();
                let y: f64 = self.rng.gen();

                let dir = &self.left_top
                    + (self.pixel_resolution * (u as f64 + x)) * &self.camera_right
                    - (self.pixel_resolution * (v as f64 + y)) * &self.camera_up;
                Ray::new(self.camera_position.clone(), dir)
            })
            .collect();

        Some((u, v, samples))
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
