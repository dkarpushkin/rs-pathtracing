use std::{sync::{Arc, Condvar, Mutex, RwLock, mpsc::{Receiver, Sender}}, thread::{spawn, JoinHandle}};

use itertools::Itertools;

use crate::{
    algebra::Vector3d,
    camera::{Camera, MultisamplerRayCaster},
    world::{Ray, World},
};

pub mod step_by_step;
pub mod thread_pool;
pub mod thread_pool_new;
pub mod threaded;

pub fn ray_color(world: &World, ray: &Ray, depth: u32) -> Vector3d {
    match world.closest_hit(&ray) {
        Some(hit) => {
            if depth == 0 {
                Vector3d::new(0.0, 0.0, 0.0)
            } else {
                let random_vector = Vector3d::get_random_unit();
                // let target = &hit.point + &hit.normal + random_vector;
                let ray_color = ray_color(
                    world,
                    &Ray {
                        origin: hit.point.clone(),
                        // direction: &target - &hit.point,
                        direction: &hit.normal + &random_vector,
                    },
                    depth - 1,
                );
                0.5 * ray_color
            }
            // 0.5 * (hit.normal.normalize() + Vector3d::new(1.0, 1.0, 1.0))
        }
        None => {
            let unit_vector = ray.direction.normalize();
            let t = 0.5 * (unit_vector.y + 1.0);
            (1.0 - t) * Vector3d::new(1.0, 1.0, 1.0) + t * Vector3d::new(0.5, 0.7, 1.0)
        }
    }
}

pub fn render_to(world: &World, camera: &Camera, buffer: &mut [u8]) {
    for (x, y, samples) in MultisamplerRayCaster::new(camera, 5) {
        // for (x, y, ray) in SinglesamplerRayCaster::new(&camera) {
        let sampled_colors = samples.iter().map(|ray| ray_color(world, ray, 20));
        let len = sampled_colors.len();
        let color: Vector3d = sampled_colors.sum::<Vector3d>() / len as f64;

        // let color = ray_color(world, &ray, 10);

        let index = ((y * camera.image().width + x) * 4) as usize;
        buffer[index + 0] = (color.x * 255.0) as u8;
        buffer[index + 1] = (color.y * 255.0) as u8;
        buffer[index + 2] = (color.z * 255.0) as u8;
        buffer[index + 3] = 255;
    }
}

pub trait Renderer {
    fn start_rendering(&mut self, camera: Arc<RwLock<Camera>>);
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

    spawn(move || {
        let rays = MultisamplerRayCaster::new(&*camera.read().unwrap(), samples_number);
        for chunk in &rays.chunks(chunk_size) {
            let chunk_vec = chunk
                .map(|(x, y, rays)| (x + y * width, rays))
                .collect_vec();
            input_sender.lock().unwrap().send(Some(chunk_vec)).unwrap();
        }
        for _ in 0..threads_num {
            input_sender.lock().unwrap().send(None).unwrap();
        }
        // println!("Dispatcher finished");
    })
}

fn new_worker_thread(
    thread_id: u32,
    input_receiver: Arc<Mutex<Receiver<InputDataVecOption>>>,
    output_sender: Arc<Mutex<Sender<OutputDataVecOption>>>,
    world: Arc<RwLock<World>>,
    parking: Arc<(Mutex<bool>, Condvar)>,
    depth: u32,
) -> JoinHandle<()> {
    spawn(move || {
        // let mut wait_time = time::Duration::microseconds(0);
        // let mut process_time = time::Duration::microseconds(0);
        // let mut process_units = 0;
        let (lock, cvar) = &*parking;
        let world = &*world.read().unwrap();
        loop {
            // let start = time::Instant::now();
            let input = match input_receiver.lock().unwrap().recv() {
                Ok(v) => v,
                Err(_) => {
                    println!("Thread {} is stopping", thread_id);
                    break;
                } // Err(mpsc::TryRecvError::Empty) => continue,
                  // Err(mpsc::TryRecvError::Disconnected) => {
                  //     println!("Thread {} is stopping", thread_id);
                  //     break;
                  // }
            };
            // let end = time::Instant::now();
            // wait_time += end - start;

            match input {
                Some(v) => {
                    // let start = time::Instant::now();
                    let result = v
                        .iter()
                        .map(|(index, rays)| {
                            let samples_colors =
                                rays.iter().map(|ray| ray_color(world, ray, depth));
                            let ln = samples_colors.len() as f64;
                            (*index, samples_colors.sum::<Vector3d>() / ln)
                        })
                        .collect_vec();
                    // let end = time::Instant::now();
                    // process_time += end - start;

                    // let start = time::Instant::now();
                    output_sender.lock().unwrap().send(Some(result)).unwrap();
                    // let end = time::Instant::now();
                    // wait_time += end - start;
                }
                None => {
                    // println!("Sent None");
                    output_sender.lock().unwrap().send(None).unwrap();
                    // println!(
                    //     "Thread {}; Processing time: {}; Wait time: {}; Units: {}",
                    //     thread_id,
                    //     process_time.whole_milliseconds(),
                    //     wait_time.whole_milliseconds(),
                    //     process_units
                    // );
                    // process_time = time::Duration::microseconds(0);
                    // wait_time = time::Duration::microseconds(0);
                    // process_units = 0;

                    let running = lock.lock().unwrap();
                    cvar.wait(running).unwrap();
                }
            }
        }
    })
}
