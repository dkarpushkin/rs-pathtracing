use crate::{
    algebra::Vector3d,
    camera::{
        ray_caster::{ImageParams, MultisamplerRayCaster},
        Camera,
    },
    world::{ray::Ray, Scene},
};
use itertools::Itertools;
use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Condvar, Mutex, RwLock,
    },
    thread::{spawn, JoinHandle},
};

pub mod step_by_step;
pub mod thread_pool;
pub mod thread_pool_new;
pub mod threaded;

pub fn ray_color(world: &Scene, ray: &Ray, depth: u32) -> Vector3d {
    match world.closest_hit(&ray, 0.001, f64::INFINITY) {
        Some(ray_hit) => {
            if depth == 0 {
                Vector3d::new(0.0, 0.0, 0.0)
            } else {
                if let Some(scatter) = ray_hit.material.scatter(ray, &ray_hit) {
                    scatter
                        .attenuation
                        .product(&ray_color(world, &scatter.ray, depth - 1))
                } else {
                    ray_hit
                        .material
                        .emitted(ray_hit.u, ray_hit.v, &ray_hit.point)
                }
            }
            // 0.5 * (ray_hit.normal.normalize() + Vector3d::new(1.0, 1.0, 1.0))
        }
        None => {
            // let t = 0.5 * (ray.direction.y + 1.0);
            // (1.0 - t) * Vector3d::new(1.0, 1.0, 1.0) + t * Vector3d::new(0.5, 0.7, 1.0)
            world.background()
        }
    }
}

pub trait Renderer {
    fn start_rendering(
        &mut self,
        camera: Arc<RwLock<Camera>>,
        img_params: &ImageParams,
        samples_number: u32,
    );
    fn render_step(&mut self, buffer: &mut Vec<Vector3d>) -> bool;
    fn stop_rendering(&mut self);
}

type InputData = (u32, Vec<Ray>);
type InputDataVec = Vec<InputData>;
type InputDataVecOption = Option<InputDataVec>;

type OutputData = (u32, Vector3d);
type OutputDataVec = Vec<OutputData>;
type OutputDataVecOption = Option<OutputDataVec>;

fn new_dispatcher_thread(
    camera: Arc<RwLock<Camera>>,
    width: u32,
    height: u32,
    samples_number: u32,
    input_sender: Arc<Mutex<Sender<InputDataVecOption>>>,
    threads_num: u32,
) -> JoinHandle<()> {
    let chunk_size = ((width * height) / threads_num / 8) as usize;
    let img_params = ImageParams { width, height };

    spawn(move || {
        let rays =
            MultisamplerRayCaster::new(&*camera.read().unwrap(), &img_params, samples_number);
        for chunk in &rays.chunks(chunk_size) {
            let chunk_vec = chunk
                .map(|(x, y, rays)| (x + y * width, rays))
                .collect_vec();
            input_sender.lock().unwrap().send(Some(chunk_vec)).unwrap();
        }
        for _ in 0..threads_num {
            input_sender.lock().unwrap().send(None).unwrap();
        }
    })
}

fn new_worker_thread(
    thread_id: u32,
    input_receiver: Arc<Mutex<Receiver<InputDataVecOption>>>,
    output_sender: Arc<Mutex<Sender<OutputDataVecOption>>>,
    world: Arc<RwLock<Scene>>,
    parking: Arc<(Mutex<bool>, Condvar)>,
    depth: u32,
) -> JoinHandle<()> {
    spawn(move || {
        let (lock, cvar) = &*parking;
        let world = &*world.read().unwrap();
        loop {
            let input = match input_receiver.lock().unwrap().recv() {
                Ok(v) => v,
                Err(_) => {
                    println!("Thread {} is stopping", thread_id);
                    break;
                }
            };
            match input {
                Some(v) => {
                    let result = trace_pixel_samples_group(v, world, depth);
                    output_sender.lock().unwrap().send(Some(result)).unwrap();
                }
                None => {
                    output_sender.lock().unwrap().send(None).unwrap();

                    let running = lock.lock().unwrap();
                    cvar.wait(running).unwrap();
                }
            }
        }
    })
}

pub fn trace_pixel_samples_group(input: InputDataVec, world: &Scene, depth: u32) -> OutputDataVec {
    // let mut result = Vec::with_capacity(input.len());
    // for (index, rays) in input {
    //     let ln = rays.len() as f64;
    //     let mut color_sum = Vector3d::zero();
    //     for ray in rays {
    //         let col = ray_color(world, &ray, depth);
    //         color_sum += col;
    //     }
    //     result.push((index, color_sum / ln))
    // }
    // result

    input
        .iter()
        .map(|input_data| {
            trace_pixel_samples(input_data, world, depth)
            // let samples_colors = rays.iter().map(|ray| ray_color(world, ray, depth));
            // let ln = samples_colors.len() as f64;
            // (*index, samples_colors.sum::<Vector3d>() / ln)
        })
        .collect_vec()
}

pub fn trace_pixel_samples(input: &InputData, world: &Scene, depth: u32) -> OutputData {
    let samples_colors = input.1.iter().map(|ray| ray_color(world, ray, depth));
    let ln = samples_colors.len() as f64;
    (input.0, samples_colors.sum::<Vector3d>() / ln)
}
