use raylib::{ffi::LoadTextureFromImage, prelude::*};
use std::{
    env, fs,
    sync::{Arc, RwLock},
};

use log::error;
use ray_tracing::{
    algebra::Vector3d,
    camera::{
        ray_caster::{ImageParams, MultisamplerRayCaster},
        Camera, CameraOrbitControl,
    },
    renderer::{step_by_step, thread_pool_new, Renderer},
    world::Scene,
};

const SIZE: (i32, i32) = (1600, 900);
// const SIZE: (u32, u32) = (800, 450);

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SIZE.0, SIZE.1)
        .title("Hello, World")
        .build();

    let args = env::args().collect::<Vec<String>>();
    let world_file = match args.len() {
        2 => &args[1],
        _ => {
            println!("Need world file");
            return;
        }
    };
    let mut state = RendererState::new(&world_file, RenderMode::StepByStep);

    let mut frame = vec![0; (SIZE.0 * SIZE.1 * 4) as usize];

    let img = Image::gen_image_cellular(SIZE.0, SIZE.1, 100);
    let mut txt = rl
        .load_texture_from_image(&thread, &img)
        .expect("could not load texture from image");

    while !rl.window_should_close() {
        state.process_input(&rl);

        state.render(&mut frame);
        txt.update_texture(&frame);

        {
            let mut d = rl.begin_drawing(&thread);

            d.clear_background(Color::BLACK);
            d.draw_texture(&txt, 0, 0, Color::WHITE);
            d.draw_fps(12, 12);
        }
    }
}

enum RenderMode {
    Static,
    StepByStep,
}

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
}

impl RendererState {
    fn new(world_file: &str, render_mode: RenderMode) -> Self {
        let json_file =
            fs::read_to_string(world_file).expect("Something went wrong reading the file");
        let scene = Scene::from_json(&json_file)
            .or_else(|err| {
                error!("Loading world failed: {}", err);
                Err(err)
            })
            .unwrap();
        // world.ad_random_spheres(50);

        let color_buffer = vec![Vector3d::new(0.0, 0.0, 0.0); (SIZE.0 * SIZE.1) as usize];
        let shared_camera = Arc::new(RwLock::new(scene.camera.clone()));
        let shared_scene = Arc::new(RwLock::new(scene));
        let renderer: Box<dyn Renderer> = match render_mode {
            RenderMode::Static => Box::new(thread_pool_new::ThreadPoolRenderer::new(
                shared_scene.clone(),
                12,
                50,
            )),
            RenderMode::StepByStep => Box::new(step_by_step::ThreadPoolRenderer::new(
                shared_scene.clone(),
                12,
                50,
            )),
        };

        let camera_control =
            CameraOrbitControl::from_camera(shared_camera.clone(), Vector3d::new(0.0, 0.0, 0.0));
        Self {
            is_redraw: true,
            is_finished: true,
            renderer: renderer,
            color_buffer,
            img_params: ImageParams {
                width: SIZE.0 as u32,
                height: SIZE.1 as u32,
            },
            shared_camera,
            shared_world: shared_scene,
            samples_num: 0,
            render_mode,
            camera_control,
            is_high_sampling: false,
        }
    }

    fn render(&mut self, frame: &mut [u8]) {
        // println!("Render");
        let samples = if self.is_high_sampling { 100 } else { 1 };

        if self.is_redraw && self.is_finished {
            self.is_redraw = false;
            self.is_finished = false;
            self.renderer.stop_rendering();
            self.renderer
                .start_rendering(self.shared_camera.clone(), &self.img_params, samples);
        }
        if !self.is_finished {
            // let start = time::Instant::now();
            self.is_finished = self.renderer.render_step(&mut self.color_buffer);
            // println!(
            //     "Rendered in {} ms",
            //     (time::Instant::now() - start).whole_milliseconds()
            // );
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

    fn process_input(&mut self, input: &RaylibHandle) -> bool {
        let is_redrawn = self.is_redraw;

        if input.is_key_down(KeyboardKey::KEY_A) {
            self.camera_control.rotate_horizontal(-0.005);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_D) {
            self.camera_control.rotate_horizontal(0.005);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_W) {
            self.camera_control.rotate_vertical(-0.005);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_S) {
            self.camera_control.rotate_vertical(0.005);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            self.camera_control.move_towards(-0.01);
            self.is_redraw = true;
        }
        if input.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
            self.camera_control.move_towards(0.01);
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

        if input.is_key_pressed(KeyboardKey::KEY_R) {
            let json = self.shared_world.read().unwrap().to_json();
            fs::write("saved_world.json", json).expect("Could not save world file");
        }

        if input.is_key_pressed(KeyboardKey::KEY_SPACE) {
            if self.is_high_sampling {
                self.is_high_sampling = false;
            } else {
                self.is_high_sampling = true;
                self.is_redraw = true;
            }
        }

        if input.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
            // if let Some((mouse_x, mouse_y)) = input.mouse() {
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
                    (index, rays),
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
}
