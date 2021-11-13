use std::{
    env, fs,
    sync::{Arc, RwLock},
};

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use ray_tracing::{
    algebra::Vector3d,
    camera::{Camera, ImageParams},
    renderer::{step_by_step, thread_pool_new, Renderer},
    world::World,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const SIZE: (u32, u32) = (1600, 900);
// const SIZE: (u32, u32) = (640, 280);
// const BOX_SIZE: i16 = 64;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(SIZE.0 as f64, SIZE.1 as f64);
        WindowBuilder::new()
            .with_title("Ray Tracing Rust")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        // let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(SIZE.0, SIZE.1, surface_texture)?;

        pixels
    };

    let args = env::args().collect::<Vec<String>>();
    let world_file = match args.len() {
        2 => &args[1],
        _ => {
            println!("Need world file");
            return Ok(());
        }
    };
    let mut state = RendererState::new(&world_file, RenderMode::StepByStep);

    event_loop.run(move |event, _, control_flow| {
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::F) {
                let size = window.inner_size();
                // let t: time::OffsetDateTime = std::time::SystemTime::now().into();
                // println!("images/rendered_{}.png", t);
                image::save_buffer(
                    format!("images/rendered.png"),
                    pixels.get_frame(),
                    size.width,
                    size.height,
                    image::ColorType::Rgba8,
                )
                .unwrap();
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                println!("Resized");
                pixels.resize_surface(size.width, size.height);
                window.request_redraw();
            }

            if state.process_input(&input) {
                println!("User input");
                window.request_redraw();
            }
            window.request_redraw();
        }

        match event {
            // Event::WindowEvent {
            //     ref event,
            //     window_id,
            // } if window_id == window.id() => match event {
            //     WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            //     WindowEvent::Resized(new_size) => {
            //         pixels.resize_buffer(new_size.width, new_size.height);
            //         pixels.resize_surface(new_size.width, new_size.height);
            //         camera.set_image(ImageParams {
            //             width: new_size.width,
            //             height: new_size.height,
            //         });
            //     }
            //     _ => {} // Update egui inputs
            //             // framework.handle_event(&event);
            // },

            // Draw the current frame
            // Event::RedrawRequested(_) => {
            //     let frame = pixels.get_frame();

            //     if redraw {
            //         color_buffer.fill(Vector3d::new(0.0, 0.0, 0.0));
            //         renderer.reset();
            //         redraw = false;
            //         samples_num = 0;
            //     }
            //     // let camera = shared_camera.read().unwrap();

            //     // let start = time::Instant::now();
            //     // let world = shared_world.read().unwrap();
            //     renderer.render_step_to(shared_camera.clone(), &mut color_buffer);
            //     // println!("Rendered for camera:\n{}", camera);
            //     // println!(
            //     //     "Camera vectors dots:\ndir to right: {}\ndir to up: {}\nup to right: {}",
            //     //     camera.direction() * camera.rigth(),
            //     //     camera.direction() * camera.up(),
            //     //     camera.up() * camera.rigth(),
            //     // );
            //     // println!(
            //     //     "Rendered in {} ms",
            //     //     (time::Instant::now() - start).whole_milliseconds()
            //     // );

            //     samples_num += 1;
            //     for (dest, src) in frame.chunks_mut(4).zip(&color_buffer) {
            //         let x = src.x;// / samples_num as f64;
            //         let y = src.y;// / samples_num as f64;
            //         let z = src.z;// / samples_num as f64;
            //         dest[0] = (x.clamp(0.0, 0.999) * 256.0) as u8;
            //         dest[1] = (y.clamp(0.0, 0.999) * 256.0) as u8;
            //         dest[2] = (z.clamp(0.0, 0.999) * 256.0) as u8;
            //         dest[3] = 255;
            //     }

            //     let render_result = pixels.render();

            //     // Basic error handling
            //     if render_result
            //         .map_err(|e| error!("pixels.render() failed: {}", e))
            //         .is_err()
            //     {
            //         *control_flow = ControlFlow::Exit;
            //     }
            // }
            Event::RedrawRequested(_) => {
                let frame = pixels.get_frame();

                state.render(frame);

                // Basic error handling
                if pixels
                    .render()
                    .map_err(|e| error!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });
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
    shared_camera: Arc<RwLock<Camera>>,
    shared_world: Arc<RwLock<World>>,
    samples_num: u32,
    render_mode: RenderMode,
}

impl RendererState {
    fn new(world_file: &str, render_mode: RenderMode) -> Self {
        let json_file =
            fs::read_to_string(world_file).expect("Something went wrong reading the file");
        let world = World::from_json(&json_file)
            .map_err(|err| {
                error!("Loading world failed: {}", err);
                Error::UserDefined(Box::new(err))
            })
            .unwrap();

        // right: (0.8783756394315214, 0, -0.4779709573324155)
        let camera = Camera::new(
            // &Vector3d::new(0.2, -0.2, 0.3),
            &Vector3d::new(0.0, 0.0, 0.0),
            &Vector3d::new(0.0, 0.0, -1.0),
            // &Vector3d::new(-0.4755988783860254, 0.09950371902099893, -0.8740164282088437),
            &Vector3d::new(0.0, 1.0, 0.0),
            // &Vector3d::new(0.047559887838602544, 0.9950371902099893, 0.08740164282088438),
            &ImageParams {
                width: SIZE.0,
                height: SIZE.1,
            },
            1.0,
            (90.0 as f64).to_radians(),
        );

        let color_buffer = vec![Vector3d::new(0.0, 0.0, 0.0); (SIZE.0 * SIZE.1) as usize];
        let shared_world = Arc::new(RwLock::new(world));
        let renderer: Box<dyn Renderer> = match render_mode {
            RenderMode::Static => Box::new(thread_pool_new::ThreadPoolRenderer::new(
                shared_world.clone(),
                12,
                50,
            )),
            RenderMode::StepByStep => Box::new(step_by_step::ThreadPoolRenderer::new(
                shared_world.clone(),
                12,
                50,
            )),
        };

        Self {
            is_redraw: true,
            is_finished: true,
            renderer: renderer,
            color_buffer,
            shared_camera: Arc::new(RwLock::new(camera)),
            shared_world: shared_world,
            samples_num: 0,
            render_mode,
        }
    }

    fn render(&mut self, frame: &mut [u8]) {
        // println!("Render");

        if self.is_redraw && self.is_finished {
            self.is_redraw = false;
            self.is_finished = false;
            self.renderer.stop_rendering();
            self.renderer.start_rendering(self.shared_camera.clone());
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

    fn process_input(&mut self, input: &WinitInputHelper) -> bool {
        let is_redrawn = self.is_redraw;

        if input.key_pressed(VirtualKeyCode::A) {
            self.shared_camera.write().unwrap().transfer(0.0, 0.1, 0.0);
            self.is_redraw = true;
        }
        if input.key_pressed(VirtualKeyCode::D) {
            self.shared_camera.write().unwrap().transfer(0.0, -0.1, 0.0);
            self.is_redraw = true;
        }
        if input.key_pressed(VirtualKeyCode::W) {
            self.shared_camera.write().unwrap().transfer(0.1, 0.0, 0.0);
            self.is_redraw = true;
        }
        if input.key_pressed(VirtualKeyCode::S) {
            self.shared_camera.write().unwrap().transfer(-0.1, 0.0, 0.0);
            self.is_redraw = true;
        }
        if input.key_pressed(VirtualKeyCode::LShift) {
            self.shared_camera.write().unwrap().transfer(0.0, 0.0, 0.1);
            self.is_redraw = true;
        }
        if input.key_pressed(VirtualKeyCode::LControl) {
            self.shared_camera.write().unwrap().transfer(0.0, 0.0, -0.1);
            self.is_redraw = true;
        }

        if input.key_pressed(VirtualKeyCode::Left) {
            self.shared_camera.write().unwrap().rotate_local(0.0, -0.1);
            // self.shared_camera.write().unwrap().rotate_global(0.1, 0.0, 0.0);
            self.is_redraw = true;
        }
        if input.key_pressed(VirtualKeyCode::Right) {
            self.shared_camera.write().unwrap().rotate_local(0.0, 0.1);
            // self.shared_camera.write().unwrap().rotate_global(-0.1, 0.0, 0.0);
            self.is_redraw = true;
        }
        if input.key_pressed(VirtualKeyCode::Up) {
            self.shared_camera.write().unwrap().rotate_local(0.1, 0.0);
            // self.shared_camera.write().unwrap().rotate_global(0.0, 0.1, 0.0);
            self.is_redraw = true;
        }
        if input.key_pressed(VirtualKeyCode::Down) {
            self.shared_camera.write().unwrap().rotate_local(-0.1, 0.0);
            // self.shared_camera.write().unwrap().rotate_global(0.0, -0.1, 0.0);
            self.is_redraw = true;
        }

        self.is_redraw ^ is_redrawn
    }
}
