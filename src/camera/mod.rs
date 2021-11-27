use std::{
    f64::consts::PI,
    fmt::Display,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::algebra::Vector3d;

pub mod ray_caster;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CameraJson {
    position: Vector3d,
    direction: Vector3d,
    up: Vector3d,
    fov: f64,
    focal_length: f64,
}

impl From<Camera> for CameraJson {
    fn from(cam: Camera) -> Self {
        CameraJson {
            position: cam.position,
            direction: cam.direction,
            up: cam.up,
            focal_length: cam.focal_length,
            fov: cam.fov.to_degrees(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(from = "CameraJson", into = "CameraJson")]
pub struct Camera {
    //  user defined
    position: Vector3d,
    direction: Vector3d,
    up: Vector3d,
    fov: f64,
    focal_length: f64,

    //  autogenerated
    rigth: Vector3d,
    // aspect_ratio: f64,
    // viewport_width: f64,
    // viewport_height: f64,
}

impl From<CameraJson> for Camera {
    fn from(cam: CameraJson) -> Self {
        Camera::new(
            &cam.position,
            &cam.direction,
            &cam.up,
            cam.focal_length,
            cam.fov.to_radians(),
        )
    }
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
        // img_params: &ImageParams,
        focal_length: f64,
        fov: f64,
    ) -> Self {
        let right_vec = direction.cross(up_vector).normalize();
        Self {
            position: position.clone(),
            direction: direction.normalize(),
            up: right_vec.cross(direction).normalize(),
            rigth: right_vec,
            fov: fov,
            focal_length: focal_length,
        }
    }

    // /// Get a reference to the camera's image.
    // pub fn image(&self) -> &ImageParams {
    //     &self.image
    // }

    // /// Set the camera's image params and change the aspect ratio.
    // pub fn set_image(&mut self, image: ImageParams) {
    //     self.aspect_ratio = image.width as f64 / image.height as f64;
    //     self.viewport_height = self.viewport_width / self.aspect_ratio;
    //     self.image = image;
    // }

    /// Get a reference to the camera's fov.
    pub fn fov(&self) -> f64 {
        self.fov
    }

    /// Set the camera's fov.
    pub fn set_fov(&mut self, fov: f64) {
        self.fov = fov;
    }

    /// Get a reference to the camera's focal length.
    pub fn focal_length(&self) -> f64 {
        self.focal_length
    }

    /// Set the camera's focal length.
    pub fn set_focal_length(&mut self, focal_length: f64) {
        self.focal_length = focal_length;
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

    /// Get a reference to the camera's rigth.
    pub fn rigth(&self) -> &Vector3d {
        &self.rigth
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

pub struct CameraOrbitControl {
    camera: Arc<RwLock<Camera>>,
    phi: f64,
    theta: f64,
    object: Vector3d,
    distance: f64,
}

impl CameraOrbitControl {
    pub fn new(
        camera: Arc<RwLock<Camera>>,
        phi: f64,
        theta: f64,
        object: Vector3d,
        distance: f64,
    ) -> Self {
        let result = Self {
            camera,
            phi,
            theta,
            object,
            distance,
        };
        result.lookat();

        result
    }

    pub fn from_camera(camera: Arc<RwLock<Camera>>, object: Vector3d) -> Self {
        let (phi, theta, distance) = {
            let cam = camera.write().unwrap();
            let pos = cam.position();
            let dir = &object - pos;
            let distance = dir.length();
            let theta = ((pos.y - object.z) / distance).acos();
            let phi = ((pos.z - object.y) / distance).atan2((pos.x - object.x) / distance);
            (phi, theta, distance)
        };

        let slf = Self {
            camera: camera.clone(),
            phi,
            theta,
            object,
            distance,
        };

        slf.lookat();

        slf
    }

    pub fn lookat(&self) {
        let pos = Vector3d::new(
            self.object.x + self.distance * self.theta.sin() * self.phi.cos(),
            self.object.z + self.distance * self.theta.cos(),
            self.object.y + self.distance * self.theta.sin() * self.phi.sin(),
        );
        let dir = &self.object - &pos;

        let mut cam = self.camera.write().unwrap();
        // println!("dir: {}", dir);
        // println!("Camera state:\n{}", cam);
        // let up = cam.rigth().cross(&dir);
        cam.set_up(Vector3d::new(0.0, 1.0, 0.0));
        cam.set_direction(dir);
        cam.set_position(pos);

        // println!("Theta: {}; Phi: {}", self.theta, self.phi);
        // println!("Camera state:\n{}", cam);
    }

    pub fn rotate_horizontal(&mut self, frac: f64) {
        self.phi += frac * PI;
        if self.phi > 2.0 * PI {
            self.phi -= 2.0 * PI;
        }
        if self.phi < 0.0 {
            self.phi += 2.0 * PI;
        }

        self.lookat();
    }

    pub fn rotate_vertical(&mut self, frac: f64) {
        self.theta += frac * PI;

        if self.theta > PI {
            self.theta = PI;
        } else if self.theta < 0.0 {
            self.theta = 0.0;
        }

        self.lookat();
    }

    pub fn move_towards(&mut self, frac: f64) {
        self.distance += frac * self.distance;

        self.lookat();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        algebra::{self, Vector3d},
        camera::ray_caster::{ImageParams, MultisamplerRayCaster},
    };

    use super::*;

    #[test]
    fn test_camera() {
        let img_params = ImageParams {
            width: 1920,
            height: 1080,
        };

        let cam = Camera::new(
            &Vector3d::new(0.0, 0.0, 0.0),
            &Vector3d::new(0.0, 0.0, -1.0),
            &Vector3d::new(0.0, 1.0, 0.0),
            1.0,
            (90.0 as f64).to_radians(),
        );

        assert_eq!(cam.rigth(), Vector3d::new(1.0, 0.0, 0.0));

        let ray_caster = MultisamplerRayCaster::new(&cam, &img_params, 100);

        assert!(algebra::approx_equal(
            ray_caster.pixel_resolution(),
            2.0 / img_params.width as f64
        ));

        assert_eq!(
            ray_caster.len(),
            (img_params.width * img_params.height) as usize
        );
    }
}
