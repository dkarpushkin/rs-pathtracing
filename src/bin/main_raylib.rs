use raylib::prelude::*;
use std::{
    env, fs,
    sync::{Arc, RwLock},
};
use time::{Duration, Instant};

use log::error;
use ray_tracing::{
    algebra::Vector3d,
    camera::{
        ray_caster::{ImageParams, MultisamplerRayCaster},
        Camera, CameraOrbitControl,
    },
    renderer::{Renderer, RenderMode, new_renderer},
    world::Scene,
};

const SIZE: (i32, i32) = (1600, 900);

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let (world_file, samples, width, height) = match args.len() {
        2 => (&args[1], 100, SIZE.0, SIZE.1),
        3 => (
            &args[1],
            args[2].parse().expect("Incorrect samples number"),
            SIZE.0,
            SIZE.1,
        ),
        5 => (
            &args[1],
            args[2].parse().expect("Incorrect samples number"),
            args[3].parse().expect("Incorrect width"),
            args[3].parse().expect("Incorrect height"),
        ),
        _ => {
            println!("Need world file");
            return;
        }
    };

    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Hello, World")
        .build();

    let mut state = RendererState::new(
        &world_file,
        RenderMode::StepByStep,
        samples,
        width as u32,
        height as u32,
    );

    let mut frame = vec![0; (width * height * 4) as usize];

    let img = Image::gen_image_cellular(width, height, 100);
    let mut txt = rl
        .load_texture_from_image(&thread, &img)
        .expect("could not load texture from image");

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            // let t: time::OffsetDateTime = std::time::SystemTime::now().into();
            // println!("images/rendered_{}.png", t);
            image::save_buffer(
                format!("images/rendered.png"),
                &frame,
                width as u32,
                height as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }

        if rl.is_cursor_on_screen() {
            rl.set_window_title(
                &thread,
                &format!(
                    "Ray Tracing Rust ({}, {})",
                    rl.get_mouse_x(),
                    rl.get_mouse_y()
                ),
            );
        }

        // if rl.is_window_resized() {
        //     let width = rl.get_screen_width();
        //     let height = rl.get_screen_height();
        //     frame = vec![0; (width * height * 4) as usize];
        //     img.resize(width, height);
        //     txt = rl.load_texture_from_image(&thread, &img).expect("could not resize texture");
        //     state.resize(width as u32, height as u32);
        // }

        state.process_input(&rl);

        state.render(&mut frame);
        txt.update_texture(&frame);

        {
            let mut d = rl.begin_drawing(&thread);

            d.clear_background(Color::BLACK);
            d.draw_texture(&txt, 0, 0, Color::WHITE);
            d.draw_fps(12, 12);
            d.draw_text(
                &format!("{} ms", state.render_duration.whole_milliseconds()),
                12,
                32,
                20,
                Color::BLACK,
            )
        }
    }
}

#[allow(dead_code)]
struct RendererState {
    is_redraw: bool,
    is_finished: bool,
    renderer: Box<dyn Renderer>,
    color_buffer: Vec<Vector3d>,
    img_params: ImageParams,
    shared_camera: Arc<RwLock<Camera>>,
    shared_world: Arc<RwLock<Scene>>,
    samples_num: u32,
    render_mode: RenderMode,

    camera_control: CameraOrbitControl,
    is_high_sampling: bool,
    samples_high: u32,

    render_start: Instant,
    render_duration: Duration,
}

impl RendererState {
    fn new(
        world_file: &str,
        render_mode: RenderMode,
        samples: u32,
        width: u32,
        height: u32,
    ) -> Self {
        let json_file =
            fs::read_to_string(world_file).expect("Something went wrong reading the file");

        let scene = Scene::from_json(&json_file)
            .or_else(|err| {
                error!("Loading world failed: {}", err);
                Err(err)
            })
            .unwrap();
        // scene.generate_cubes(20);
        // scene.add_random_spheres();

        let color_buffer = vec![Vector3d::new(0.0, 0.0, 0.0); (width * height) as usize];
        let shared_camera = Arc::new(RwLock::new(scene.camera().clone()));
        let shared_scene = Arc::new(RwLock::new(scene));
        let renderer: Box<dyn Renderer> = new_renderer(render_mode, shared_scene.clone());

        let camera_control = CameraOrbitControl::from_camera(
            shared_camera.clone(),
            // Vector3d::new(278.0, 278.0, 0.0),
            Vector3d::new(0.0, 0.0, 0.0),
        );
        Self {
            is_redraw: true,
            is_finished: true,
            renderer: renderer,
            color_buffer,
            img_params: ImageParams {
                width: width as u32,
                height: height as u32,
            },
            shared_camera,
            shared_world: shared_scene,
            samples_num: 0,
            render_mode,
            camera_control,

            is_high_sampling: false,
            samples_high: samples,

            render_start: Instant::now(),
            render_duration: Duration::seconds(0),
        }
    }

    fn render(&mut self, frame: &mut [u8]) {
        let samples_number = if self.is_high_sampling {
            self.samples_high
        } else {
            1
        };

        if self.is_redraw && self.is_finished {
            self.is_redraw = false;
            self.is_finished = false;
            self.renderer.stop_rendering();
            self.renderer.start_rendering(
                self.shared_camera.clone(),
                &self.img_params,
                samples_number,
            );
            self.render_start = Instant::now();

            // for v in self.color_buffer.iter_mut() {
            //     *v = Vector3d::zero();
            // }
        }

        if !self.is_finished {
            self.is_finished = self.renderer.render_step(&mut self.color_buffer);

            if self.is_finished {
                self.render_duration = Instant::now() - self.render_start;
            }

            for (dest, src) in frame.chunks_mut(4).zip(&self.color_buffer) {
                let r = src.x.sqrt();
                let g = src.y.sqrt();
                let b = src.z.sqrt();
                dest[0] = (r.clamp(0.0, 0.999) * 256.0) as u8;
                dest[1] = (g.clamp(0.0, 0.999) * 256.0) as u8;
                dest[2] = (b.clamp(0.0, 0.999) * 256.0) as u8;
                dest[3] = 255;
            }
        }
    }

    fn process_input(&mut self, input: &RaylibHandle) -> bool {
        let is_redrawn = self.is_redraw;

        if input.is_key_down(KeyboardKey::KEY_A) {
            // self.camera_control.rotate_horizontal(-0.005);
            self.shared_camera.write().unwrap().transfer(0.0, -0.2, 0.0);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_D) {
            // self.camera_control.rotate_horizontal(0.005);
            self.shared_camera.write().unwrap().transfer(0.0, 0.2, 0.0);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_W) {
            // self.camera_control.rotate_vertical(-0.005);
            self.shared_camera.write().unwrap().transfer(0.2, 0.0, 0.0);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_S) {
            // self.camera_control.rotate_vertical(0.005);
            self.shared_camera.write().unwrap().transfer(-0.2, 0.0, 0.0);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            // self.camera_control.move_towards(-0.01);
            self.shared_camera.write().unwrap().transfer(0.0, 0.0, 0.2);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
            // self.camera_control.move_towards(0.01);
            self.shared_camera.write().unwrap().transfer(0.0, 0.0, -0.2);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_E) {
            self.shared_camera.write().unwrap().rotate_local(0.0, 0.01);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_Q) {
            self.shared_camera.write().unwrap().rotate_local(0.0, -0.01);
            self.is_redraw = true;
        }

        if input.is_key_pressed(KeyboardKey::KEY_KP_ADD) {
            let mut cam = self.shared_camera.write().unwrap();
            let old_fov = cam.fov();
            cam.set_fov(old_fov - (1.0 as f64).to_radians());
            self.is_redraw = true;
        }
        if input.is_key_pressed(KeyboardKey::KEY_KP_SUBTRACT) {
            let mut cam = self.shared_camera.write().unwrap();
            let old_fov = cam.fov();
            cam.set_fov(old_fov + (1.0 as f64).to_radians());
            self.is_redraw = true;
        }

        // if input.is_key_pressed(KeyboardKey::KEY_R) {
        //     let json = self.shared_world.read().unwrap().to_json();
        //     fs::write("saved_world.json", json).expect("Could not save world file");
        // }

        if input.is_key_pressed(KeyboardKey::KEY_SPACE) {
            if self.is_high_sampling {
                self.is_high_sampling = false;
            } else {
                self.is_high_sampling = true;
                self.is_redraw = true;
            }
        }

        if input.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
            if input.is_cursor_on_screen() {
                let mouse_x = input.get_mouse_x();
                let mouse_y = input.get_mouse_y();
                let camera = self.shared_camera.read().unwrap();
                let mut sampler = MultisamplerRayCaster::new(&camera, &self.img_params, 1);
                let rays = sampler.get_pixel_sample(mouse_x as u32, mouse_y as u32);
                let index = mouse_x as u32 + mouse_y as u32 * self.img_params.width;
                println!("({}, {})", mouse_x, mouse_y);
                println!("rays: {:?}", &rays);
                let r = ray_tracing::renderer::trace_pixel_samples(
                    &(index, rays),
                    &*self.shared_world.read().unwrap(),
                    10,
                );
                println!("{}", r.1);
            }
        }

        if self.is_redraw && !is_redrawn {
            let camera = self.shared_camera.read().unwrap();

            println!("Rendering for camera:\n{}", camera);

            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn resize(&mut self, width: u32, height: u32) {
        self.img_params = ImageParams { width, height };
        self.color_buffer = vec![Vector3d::new(0.0, 0.0, 0.0); (width * height) as usize];
        self.is_redraw = true;
    }
}
