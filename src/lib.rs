pub mod algebra;
pub mod camera;
pub mod renderer;
pub mod world;

#[cfg(test)]
mod tests {
    use crate::{algebra::Vector3d, camera::ImageParams};

    use super::*;

    #[test]
    fn test_camera() {
        let mut cam = camera::Camera::new(
            &Vector3d::new(0.0, 0.0, 0.0),
            &Vector3d::new(0.0, 0.0, -1.0),
            &Vector3d::new(0.0, 1.0, 0.0),
            &ImageParams {
                width: 1920,
                height: 1080,
            },
            1.0,
            (90.0 as f64).to_radians(),
        );

        assert!(algebra::approx_equal(cam.viewport_width(), 2.0));
        assert!(algebra::approx_equal(cam.viewport_height(), 1.125));
        assert!(algebra::approx_equal(cam.aspect_ratio(), 16.0 / 9.0));
        assert_eq!(cam.rigth(), Vector3d::new(1.0, 0.0, 0.0));

        cam.set_image(ImageParams {
            width: 16,
            height: 9,
        });
        assert!(algebra::approx_equal(cam.aspect_ratio(), 16.0 / 9.0));

        // let caster = cam.rays();
        // println!("{:?}", caster);
        // let rays = caster.collect::<Vec<(u32, u32, world::Ray)>>();
        // println!("{}", rays.len());
        // for ray in rays {
        //     println!("{:?} ", ray);
        // }
    }
}
