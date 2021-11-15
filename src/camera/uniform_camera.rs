use std::fmt::Display;

use crate::algebra::{Vector3d, matrix::Matrix4x4d};

use super::ImageParams;

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
        img_params: &ImageParams,
        focal_length: f64,
        fov: f64,
        transform: Matrix4x4d
    ) -> Self {
        let aspect_ratio = img_params.width as f64 / img_params.height as f64;
        let viewport_width = (fov / 2.0).tan() * focal_length * 2.0;
        Self {
            position: Vector3d::new(0.0, 0.0, 0.0),
            direction: Vector3d::new(0.0, 0.0, -1.0),
            up: Vector3d::new(0.0, 1.0, 0.0),
            rigth: Vector3d::new(1.0, 0.0, 0.0),
            fov: fov,
            focal_length: focal_length,
            image: img_params.clone(),
            aspect_ratio: aspect_ratio,

            viewport_width: viewport_width,
            viewport_height: viewport_width / aspect_ratio,
        }
    }
}
